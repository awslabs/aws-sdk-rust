/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Real transferred-byte counting for telemetry.
//!
//! The request and response bodies are wrapped in a counter that tallies bytes per frame as they
//! flow. `Content-Length` is not used because it is `None`/`0` for streaming bodies. Each wrapped
//! body records its total to a dedicated histogram *on completion* (when the body is dropped), so a
//! streaming body is measured once fully consumed rather than read as `0` at operation-return time.
//!
//! Body size is recorded as its own instrument (`smithy.client.call.request.size` /
//! `smithy.client.call.response.size`) rather than as an attribute on the call-duration histogram:
//! body size is near-unique per call, so attaching it as a label would fragment the duration metric
//! into one time series per byte count.

use aws_smithy_observability::instruments::Histogram;
use aws_smithy_observability::{AttributeValue, Attributes};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::{
    BeforeDeserializationInterceptorContextMut, BeforeTransmitInterceptorContextMut,
};
use aws_smithy_runtime_api::client::interceptors::Intercept;
use aws_smithy_runtime_api::client::orchestrator::Metadata;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::ConfigBag;
use http_body_1x::{Body, Frame};
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::client::metrics::OperationTelemetry;

/// Body wrapper that tallies each data frame's length and, when dropped, records the running total
/// to `instrument` with `attributes`. Contents are forwarded unchanged.
///
/// Recording on `Drop` (rather than at a fixed hook) is what lets a streaming body be measured
/// correctly: the total is emitted once the body is fully consumed and released. On a retried
/// request each attempt wraps a fresh body, so each attempt records its own measurement as it is
/// dropped — the counts are never shared or overwritten across attempts.
struct CountingBody<B> {
    inner: B,
    transferred: u64,
    instrument: Arc<dyn Histogram>,
    attributes: Attributes,
}

impl<B> CountingBody<B> {
    fn new(inner: B, instrument: Arc<dyn Histogram>, attributes: Attributes) -> Self {
        Self {
            inner,
            transferred: 0,
            instrument,
            attributes,
        }
    }
}

impl<B> Drop for CountingBody<B> {
    fn drop(&mut self) {
        self.instrument
            .record(self.transferred as f64, Some(&self.attributes), None);
    }
}

impl<B> Body for CountingBody<B>
where
    B: Body<Data = bytes::Bytes, Error = BoxError> + Unpin,
{
    type Data = bytes::Bytes;
    type Error = BoxError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = &mut *self;
        match Pin::new(&mut this.inner).poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) => {
                if let Some(data) = frame.data_ref() {
                    // Count only data frames; trailers carry no payload bytes.
                    this.transferred += data.len() as u64;
                }
                Poll::Ready(Some(Ok(frame)))
            }
            other => other,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body_1x::SizeHint {
        self.inner.size_hint()
    }
}

/// Wraps `body` so its transferred bytes are recorded to `instrument` on completion, preserving
/// contents.
fn wrap(body: SdkBody, instrument: Arc<dyn Histogram>, attributes: Attributes) -> SdkBody {
    body.map_preserve_contents(move |b| {
        SdkBody::from_body_1_x(CountingBody::new(b, instrument.clone(), attributes.clone()))
    })
}

/// The `rpc.service`/`rpc.method` attributes to record alongside a body size, if operation metadata
/// is available.
fn rpc_attributes(cfg: &ConfigBag) -> Attributes {
    let mut attrs = Attributes::new();
    if let Some(md) = cfg.load::<Metadata>() {
        attrs.set("rpc.service", AttributeValue::String(md.service().into()));
        attrs.set("rpc.method", AttributeValue::String(md.name().into()));
    }
    attrs
}

/// Wraps request and response bodies so their transferred sizes are recorded to the dedicated
/// body-size histograms on completion.
#[derive(Debug, Default)]
pub(crate) struct TelemetryBytesInterceptor;

impl Intercept for TelemetryBytesInterceptor {
    fn name(&self) -> &'static str {
        "TelemetryBytesInterceptor"
    }

    fn modify_before_transmit(
        &self,
        context: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let Some(instruments) = cfg.load::<OperationTelemetry>() else {
            return Ok(());
        };
        let instrument = instruments.request_body_size.clone();
        let attributes = rpc_attributes(cfg);

        let body = mem::replace(context.request_mut().body_mut(), SdkBody::taken());
        *context.request_mut().body_mut() = wrap(body, instrument, attributes);
        Ok(())
    }

    fn modify_before_deserialization(
        &self,
        context: &mut BeforeDeserializationInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let Some(instruments) = cfg.load::<OperationTelemetry>() else {
            return Ok(());
        };
        let instrument = instruments.response_body_size.clone();
        let attributes = rpc_attributes(cfg);

        let body = mem::replace(context.response_mut().body_mut(), SdkBody::taken());
        *context.response_mut().body_mut() = wrap(body, instrument, attributes);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use aws_smithy_observability::instruments::Histogram;
    use futures_util::StreamExt;
    use http_body_util::BodyExt;
    use std::sync::Mutex;

    // A minimal in-memory histogram that captures recorded values.
    #[derive(Debug, Default)]
    struct RecordingHistogram {
        values: Mutex<Vec<f64>>,
    }

    impl Histogram for RecordingHistogram {
        fn record(
            &self,
            value: f64,
            _attributes: Option<&Attributes>,
            _context: Option<&dyn aws_smithy_observability::Context>,
        ) {
            self.values.lock().unwrap().push(value);
        }
    }

    async fn drain(body: SdkBody) {
        let _ = body.collect().await.expect("body drains");
    }

    #[tokio::test]
    async fn records_all_bytes_of_a_streaming_body_on_completion() {
        // A streaming body has no Content-Length; the wrapper must record the real total once the
        // body is fully consumed and dropped.
        let hist = Arc::new(RecordingHistogram::default());
        let stream = futures_util::stream::iter(vec![
            Ok::<_, BoxError>(bytes::Bytes::from_static(b"hello ")),
            Ok(bytes::Bytes::from_static(b"world")),
        ]);
        let streaming = SdkBody::from_body_1_x(http_body_util::StreamBody::new(
            stream.map(|r| r.map(Frame::data)),
        ));
        assert_eq!(None, streaming.content_length(), "precondition: streaming");

        drain(wrap(streaming, hist.clone(), Attributes::new())).await;

        assert_eq!(vec![11.0], *hist.values.lock().unwrap());
    }

    #[tokio::test]
    async fn empty_body_records_zero() {
        let hist = Arc::new(RecordingHistogram::default());
        drain(wrap(SdkBody::empty(), hist.clone(), Attributes::new())).await;
        assert_eq!(vec![0.0], *hist.values.lock().unwrap());
    }

    #[tokio::test]
    async fn each_body_records_independently() {
        // Two separate wrapped bodies (as happens across retry attempts) each record their own
        // measurement — nothing is shared or overwritten.
        let hist = Arc::new(RecordingHistogram::default());
        drain(wrap(SdkBody::from("abc"), hist.clone(), Attributes::new())).await;
        drain(wrap(SdkBody::from("de"), hist.clone(), Attributes::new())).await;
        let mut recorded = hist.values.lock().unwrap().clone();
        recorded.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(vec![2.0, 3.0], recorded);
    }
}
