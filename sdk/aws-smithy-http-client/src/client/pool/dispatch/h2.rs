/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP/2 dispatch and two-ended request completion.
//!
//! [`H2Activation`] arrives with prospective request authority and a transient
//! sender cloned from the connection-owning cell. Dispatch first checks sender
//! and logical-connection state, then polls Hyper once. A request returned from
//! that poll was not accepted and may re-enter protocol acquisition.
//!
//! After Hyper accepts the request, the activation becomes an accepted claim.
//! [`H2RequestBody`] owns the upload guard and [`H2ResponseBody`] owns the
//! response guard. Either may finish first; the generation request count is
//! released only after both finish. A returned request re-arms the existing
//! body wrapper rather than nesting wrappers around the original body.

use super::super::cell::h2::{
    H2Activation, H2CloseHandle, H2DispatchParts, H2ResponseGuard, H2UploadGuard,
};
use super::super::connection::{CloseReason, ConnectionState};
use super::{AcquisitionContext, FrozenConnectionSelection, H1HostHeaderInserted};
use crate::client::connect::ConnectPathInner;
use crate::client::downcast_error;
use crate::sync::{Arc, Mutex};
use aws_smithy_runtime_api::client::connection::CaptureSmithyConnection;
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_types::body::SdkBody;
use http_1x::{Request, Response};
use hyper::body::Body;
use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Result of one checked HTTP/2 dispatch attempt.
#[allow(
    clippy::large_enum_variant,
    reason = "boxing the successful response would allocate on every HTTP/2 request"
)]
pub(super) enum H2DispatchOutcome {
    /// Hyper accepted the request and produced a guarded response.
    Response(Response<SdkBody>),
    /// Hyper did not accept the request, so it may re-enter acquisition.
    NotAccepted(Box<H2UnacceptedRequest>),
}

/// Request and terminal fallback retained for one replacement selection.
pub(super) struct H2UnacceptedRequest {
    /// Original request Hyper did not accept.
    request: Request<SdkBody>,
    /// Error returned if the request exhausts its replacement budget.
    terminal_error: ConnectorError,
}

impl H2UnacceptedRequest {
    /// Returns the original request and its terminal fallback.
    pub(super) fn into_parts(self: Box<Self>) -> (Request<SdkBody>, ConnectorError) {
        let Self {
            request,
            terminal_error,
        } = *self;
        (request, terminal_error)
    }
}

/// State retained after Hyper accepts one request.
struct H2AcceptedDispatch {
    /// Connection retained through response creation.
    connection: Arc<ConnectionState>,
    /// Generation close authority used for connection-wide failures.
    close: H2CloseHandle,
    /// Metadata captured before the request moved into Hyper.
    captured_metadata: Option<aws_smithy_runtime_api::client::connection::ConnectionMetadata>,
    /// Response guard transferred to the guarded response body.
    response_guard: H2ResponseGuard,
    /// Whether the generation accepted an earlier request.
    reused: bool,
    /// Selected-connection observation held until the request cannot return.
    connection_selection: Option<FrozenConnectionSelection>,
}

/// Dispatches one request through prospective H2 request authority.
///
/// Pool-side staleness reacquires before Hyper sees the request. Once Hyper
/// returns an envelope, only a reused generation may reacquire.
pub(super) async fn dispatch(
    context: &AcquisitionContext,
    mut request: Request<SdkBody>,
    mut activation: H2Activation,
) -> Result<H2DispatchOutcome, ConnectorError> {
    let connection = activation.connection().clone();
    prepare_h2_request(
        &mut request,
        &context.absolute_uri,
        connection.info().connect_path_inner(),
    );
    let reused = activation.is_reused();
    let close = activation.close_handle();
    let connection_capture = request
        .extensions()
        .get::<CaptureSmithyConnection>()
        .cloned();
    let captured_metadata = (connection_capture.is_some()
        || context.captures_connection_selection())
    .then(|| connection.info().h2_metadata(close.clone()));
    if let (Some(capture), Some(metadata)) = (connection_capture, &captured_metadata) {
        let captured = metadata.clone();
        capture.set_connection_retriever(move || Some(captured.clone()));
    }

    let H2DispatchParts {
        mut sender,
        upload,
        response,
    } = activation.take_dispatch_parts();
    if sender.is_closed() {
        close.close(CloseReason::ProtocolClosed);
        let metadata = captured_metadata.unwrap_or_else(|| connection.info().h2_metadata(close));
        return resolve_unaccepted_request(
            request,
            reused,
            UnacceptedStage::BeforeHyper,
            ConnectorError::other(H2ConnectionClosedBeforeDispatch.into(), None)
                .with_connection(metadata),
        );
    }
    let Some(dispatch) = ConnectionState::try_commit_dispatch(&connection) else {
        let metadata = captured_metadata.unwrap_or_else(|| connection.info().h2_metadata(close));
        return resolve_unaccepted_request(
            request,
            reused,
            UnacceptedStage::BeforeHyper,
            ConnectorError::other(H2ConnectionClosedBeforeDispatch.into(), None)
                .with_connection(metadata),
        );
    };

    let body = H2RequestBodyHandle::arm(&mut request, upload);

    let mut send = Box::pin(sender.hyper_mut().try_send_request(request));
    let first = poll_fn(|cx| Poll::Ready(send.as_mut().poll(cx))).await;

    let first = match first {
        Poll::Ready(Err(mut error)) => {
            if let Some(returned) = error.take_message() {
                body.clear();
                drop(response);
                drop(dispatch);
                drop(activation);
                close.close(CloseReason::ProtocolClosed);
                let metadata =
                    captured_metadata.unwrap_or_else(|| connection.info().h2_metadata(close));
                return resolve_unaccepted_request(
                    returned,
                    reused,
                    UnacceptedStage::ReturnedByHyper,
                    downcast_error(Box::new(error.into_error())).with_connection(metadata),
                );
            }
            Poll::Ready(Err(error))
        }
        first => first,
    };

    match first {
        Poll::Ready(result) => {
            activation.accept(dispatch);
            let connection_selection =
                context.freeze_connection_selection(reused, captured_metadata.as_ref());
            let sender_closed = sender.is_closed();
            resolve_h2_send(
                sender_closed,
                result,
                H2AcceptedDispatch {
                    connection,
                    close,
                    captured_metadata,
                    response_guard: response,
                    reused,
                    connection_selection,
                },
            )
        }
        Poll::Pending => {
            activation.accept(dispatch);
            let connection_selection =
                context.freeze_connection_selection(reused, captured_metadata.as_ref());
            let result = send.await;
            let sender_closed = sender.is_closed();
            resolve_h2_send(
                sender_closed,
                result,
                H2AcceptedDispatch {
                    connection,
                    close,
                    captured_metadata,
                    response_guard: response,
                    reused,
                    connection_selection,
                },
            )
        }
    }
}

fn resolve_h2_send(
    sender_closed: bool,
    result: Result<
        Response<hyper::body::Incoming>,
        hyper::client::conn::TrySendError<Request<SdkBody>>,
    >,
    accepted: H2AcceptedDispatch,
) -> Result<H2DispatchOutcome, ConnectorError> {
    let H2AcceptedDispatch {
        connection,
        close,
        captured_metadata,
        response_guard,
        reused,
        connection_selection,
    } = accepted;
    match result {
        Ok(mut response) => {
            if let Some(selection) = connection_selection {
                selection.record();
            }
            connection
                .info()
                .apply_connector_extras(response.extensions_mut());
            let (parts, body) = response.into_parts();
            let body = H2ResponseBody::new(body, response_guard);
            Ok(H2DispatchOutcome::Response(Response::from_parts(
                parts,
                SdkBody::from_body_1_x(body),
            )))
        }
        Err(mut error) => {
            // Hyper may return a queued envelope when its connection task
            // drops after the first poll. The returned request is the
            // authority for reacquisition; an error without it is terminal.
            if let Some(request) = error.take_message() {
                if let Some(body) = request.extensions().get::<H2RequestBodyHandle>() {
                    body.clear();
                }
                drop(response_guard);
                close.close(CloseReason::ProtocolClosed);
                let metadata =
                    captured_metadata.unwrap_or_else(|| connection.info().h2_metadata(close));
                return resolve_unaccepted_request(
                    request,
                    reused,
                    UnacceptedStage::ReturnedByHyper,
                    downcast_error(Box::new(error.into_error())).with_connection(metadata),
                );
            }
            if sender_closed {
                close.close(CloseReason::ProtocolClosed);
            }
            drop(response_guard);
            if let Some(selection) = connection_selection {
                selection.record();
            }
            let metadata =
                captured_metadata.unwrap_or_else(|| connection.info().h2_metadata(close));
            Err(downcast_error(Box::new(error.into_error())).with_connection(metadata))
        }
    }
}

/// Restores the absolute URI and applies connection-path request metadata.
fn prepare_h2_request(
    request: &mut Request<SdkBody>,
    absolute_uri: &http_1x::Uri,
    connect_path: &ConnectPathInner,
) {
    *request.uri_mut() = absolute_uri.clone();
    if request
        .extensions_mut()
        .remove::<H1HostHeaderInserted>()
        .is_some()
    {
        request.headers_mut().remove(http_1x::header::HOST);
    }
    connect_path.apply_proxy_authorization(request.headers_mut());
}

/// Point at which the pool learned that Hyper had not accepted the request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnacceptedStage {
    /// The pool rejected a stale generation before calling Hyper.
    BeforeHyper,
    /// Hyper returned the original request envelope.
    ReturnedByHyper,
}

/// Applies retry authority to a request that Hyper did not accept.
///
/// Pool-side checks always reacquire because no protocol code observed the
/// request. A returned Hyper envelope reacquires only after prior successful
/// use proves that replacing a stale pooled generation is appropriate.
fn resolve_unaccepted_request(
    request: Request<SdkBody>,
    reused: bool,
    stage: UnacceptedStage,
    error: ConnectorError,
) -> Result<H2DispatchOutcome, ConnectorError> {
    if stage == UnacceptedStage::BeforeHyper || reused {
        Ok(H2DispatchOutcome::NotAccepted(Box::new(
            H2UnacceptedRequest {
                request,
                terminal_error: error,
            },
        )))
    } else {
        Err(error)
    }
}

/// A fresh HTTP/2 generation closed before accepting its first request.
#[derive(Debug)]
struct H2ConnectionClosedBeforeDispatch;

impl std::fmt::Display for H2ConnectionClosedBeforeDispatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the fresh HTTP/2 connection closed before request dispatch")
    }
}

impl std::error::Error for H2ConnectionClosedBeforeDispatch {}

/// Handle retained in a request extension after its body is wrapped once.
#[derive(Clone)]
struct H2RequestBodyHandle {
    /// Re-armable upload guard shared with the wrapped request body.
    slot: Arc<Mutex<Option<H2UploadGuard>>>,
}

impl H2RequestBodyHandle {
    /// Wraps an unwrapped body and arms the current upload guard.
    fn arm(request: &mut Request<SdkBody>, upload: H2UploadGuard) -> Self {
        let is_end_stream = request.body().is_end_stream();
        let handle = request
            .extensions()
            .get::<Self>()
            .cloned()
            .unwrap_or_else(|| {
                let handle = Self {
                    slot: Arc::new(Mutex::new(None)),
                };
                let body = std::mem::replace(request.body_mut(), SdkBody::taken());
                *request.body_mut() = SdkBody::from_body_1_x(H2RequestBody {
                    inner: body,
                    slot: handle.slot.clone(),
                });
                request.extensions_mut().insert(handle.clone());
                handle
            });
        arm_upload_guard(&handle.slot, upload);
        if is_end_stream {
            finish_upload(&handle.slot);
        }
        handle
    }

    /// Disarms the upload guard after Hyper returns the request unaccepted.
    fn clear(&self) {
        finish_upload(&self.slot);
    }
}

/// Request body whose upload guard can be re-armed after non-acceptance.
struct H2RequestBody {
    /// Original SDK body wrapped exactly once.
    inner: SdkBody,
    /// Guard replaced when Hyper returns an unaccepted request.
    slot: Arc<Mutex<Option<H2UploadGuard>>>,
}

impl Body for H2RequestBody {
    type Data = <SdkBody as Body>::Data;
    type Error = <SdkBody as Body>::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        let result = Pin::new(&mut self.inner).poll_frame(cx);
        if matches!(result, Poll::Ready(None) | Poll::Ready(Some(Err(_)))) {
            finish_upload(&self.slot);
        }
        result
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> hyper::body::SizeHint {
        self.inner.size_hint()
    }
}

impl Drop for H2RequestBody {
    fn drop(&mut self) {
        finish_upload(&self.slot);
    }
}

/// Response body that owns the response guard through stream completion.
struct H2ResponseBody {
    /// Hyper response stream.
    inner: hyper::body::Incoming,
    /// Response guard finished on terminal frame, error, or drop.
    response: Option<H2ResponseGuard>,
}

impl H2ResponseBody {
    /// Wraps a response and finishes a guard already at end stream.
    fn new(inner: hyper::body::Incoming, response: H2ResponseGuard) -> Self {
        let is_end_stream = inner.is_end_stream();
        Self {
            inner,
            response: retain_response_guard(is_end_stream, response),
        }
    }
}

/// Retains a response guard only while response frames may remain.
fn retain_response_guard(
    is_end_stream: bool,
    response: H2ResponseGuard,
) -> Option<H2ResponseGuard> {
    if is_end_stream {
        response.finish();
        None
    } else {
        Some(response)
    }
}

impl Body for H2ResponseBody {
    type Data = <hyper::body::Incoming as Body>::Data;
    type Error = <hyper::body::Incoming as Body>::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        let result = Pin::new(&mut self.inner).poll_frame(cx);
        if matches!(result, Poll::Ready(None) | Poll::Ready(Some(Err(_)))) {
            if let Some(response) = self.response.take() {
                response.finish();
            }
        }
        result
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> hyper::body::SizeHint {
        self.inner.size_hint()
    }
}

fn arm_upload_guard(slot: &Arc<Mutex<Option<H2UploadGuard>>>, upload: H2UploadGuard) {
    let previous = slot.lock().replace(upload);
    drop(previous);
}

/// Finishes the upload guard after upload completion, error, or rejection.
fn finish_upload(slot: &Arc<Mutex<Option<H2UploadGuard>>>) {
    let upload = slot.lock().take();
    if let Some(upload) = upload {
        upload.finish();
    }
}

#[cfg(all(test, not(smithy_http_client_loom), feature = "rt-tokio"))]
mod tests {
    use super::*;
    use crate::client::pool::cell::OriginCell;
    use crate::client::pool::origin::OriginKey;
    use crate::client::pool::partition::{EligibilityGroup, PartitionId};
    use crate::sync::Arc;
    use http_1x::uri::Scheme;
    use http_body_util::BodyExt;

    /// Request body that fails on its first frame.
    struct FailingBody;

    impl Body for FailingBody {
        type Data = hyper::body::Bytes;
        type Error = std::io::Error;

        fn poll_frame(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
            Poll::Ready(Some(Err(std::io::Error::other("request body failed"))))
        }
    }

    fn cell() -> Arc<OriginCell> {
        Arc::new(OriginCell::new(
            PartitionId::from_index(1),
            OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
            EligibilityGroup::Pool,
            None,
            None,
        ))
    }

    #[tokio::test]
    async fn returned_request_rearms_one_body_wrapper() {
        let cell = cell();
        let mut request = Request::post("https://example.com/upload")
            .body(SdkBody::from("payload"))
            .unwrap();

        let (first_upload, first_probe) = H2UploadGuard::for_test(&cell);
        let first = H2RequestBodyHandle::arm(&mut request, first_upload);
        let first_slot = first.slot.clone();
        assert!(!first_probe.upload_finished());

        let (second_upload, second_probe) = H2UploadGuard::for_test(&cell);
        let second = H2RequestBodyHandle::arm(&mut request, second_upload);
        assert!(
            Arc::ptr_eq(&first_slot, &second.slot),
            "rearming replaced the request-body wrapper"
        );
        assert!(
            first_probe.upload_finished(),
            "rearming did not cancel the prior prospective upload guard"
        );
        assert!(!second_probe.upload_finished());

        second.clear();
        assert!(
            second_probe.upload_finished(),
            "returned request retained its rejected upload guard"
        );
        let body = request
            .into_body()
            .collect()
            .await
            .expect("wrapped request body failed")
            .to_bytes();
        assert_eq!(hyper::body::Bytes::from_static(b"payload"), body);
    }

    #[test]
    fn empty_request_body_finishes_its_upload_guard_when_armed() {
        let cell = cell();
        let mut request = Request::new(SdkBody::empty());
        let (upload, probe) = H2UploadGuard::for_test(&cell);

        H2RequestBodyHandle::arm(&mut request, upload);

        assert!(probe.upload_finished());
    }

    #[test]
    fn dropping_request_body_finishes_its_upload_guard() {
        let cell = cell();
        let mut request = Request::new(SdkBody::from("payload"));
        let (upload, probe) = H2UploadGuard::for_test(&cell);

        H2RequestBodyHandle::arm(&mut request, upload);
        let body = std::mem::replace(request.body_mut(), SdkBody::empty());
        drop(body);

        assert!(probe.upload_finished());
    }

    #[tokio::test]
    async fn request_body_error_finishes_its_upload_guard() {
        let cell = cell();
        let mut request = Request::new(SdkBody::from_body_1_x(FailingBody));
        let (upload, probe) = H2UploadGuard::for_test(&cell);

        H2RequestBodyHandle::arm(&mut request, upload);
        let frame = request
            .body_mut()
            .frame()
            .await
            .expect("failing body omitted its error frame");

        assert!(frame.is_err());
        assert!(probe.upload_finished());
    }

    #[test]
    fn bodyless_response_finishes_its_response_guard_without_drop() {
        let cell = cell();
        let (response, probe) = H2ResponseGuard::for_test(&cell);

        let retained = retain_response_guard(true, response);

        assert!(retained.is_none());
        assert!(probe.response_finished());
    }

    #[test]
    fn h2_preparation_restores_uri_and_removes_only_synthesized_host() {
        let absolute: http_1x::Uri = "https://example.com/resource".parse().unwrap();
        let mut request = Request::get("/resource").body(SdkBody::empty()).unwrap();
        request
            .headers_mut()
            .insert(http_1x::header::HOST, "example.com".parse().unwrap());
        request.extensions_mut().insert(H1HostHeaderInserted);

        prepare_h2_request(&mut request, &absolute, &ConnectPathInner::Direct);

        assert_eq!(&absolute, request.uri());
        assert!(!request.headers().contains_key(http_1x::header::HOST));
        assert!(request.extensions().get::<H1HostHeaderInserted>().is_none());

        let mut user_host = Request::get("/resource").body(SdkBody::empty()).unwrap();
        user_host
            .headers_mut()
            .insert(http_1x::header::HOST, "signed.example".parse().unwrap());
        prepare_h2_request(&mut user_host, &absolute, &ConnectPathInner::Direct);
        assert_eq!("signed.example", user_host.headers()[http_1x::header::HOST]);
    }

    #[test]
    fn h2_preparation_applies_forward_proxy_auth_without_overwriting_caller() {
        let absolute: http_1x::Uri = "http://example.com/resource".parse().unwrap();
        let connect_path = ConnectPathInner::forward_proxy(Some(
            http_1x::HeaderValue::from_static("Basic connector"),
        ));
        let mut request = Request::get("/resource").body(SdkBody::empty()).unwrap();

        prepare_h2_request(&mut request, &absolute, &connect_path);

        assert_eq!(
            "Basic connector",
            request.headers()[http_1x::header::PROXY_AUTHORIZATION]
        );

        let caller_value = http_1x::HeaderValue::from_static("Basic caller");
        let mut request = Request::get("/resource")
            .header(http_1x::header::PROXY_AUTHORIZATION, caller_value.clone())
            .body(SdkBody::empty())
            .unwrap();

        prepare_h2_request(&mut request, &absolute, &connect_path);

        assert_eq!(
            Some(&caller_value),
            request.headers().get(http_1x::header::PROXY_AUTHORIZATION)
        );
    }

    #[test]
    fn retry_authority_distinguishes_pool_checks_from_hyper_returns() {
        let checked_before_hyper = resolve_unaccepted_request(
            Request::new(SdkBody::empty()),
            false,
            UnacceptedStage::BeforeHyper,
            ConnectorError::user("pre-Hyper failure".into()),
        );
        assert!(matches!(
            checked_before_hyper,
            Ok(H2DispatchOutcome::NotAccepted(_))
        ));

        let fresh_hyper_return = resolve_unaccepted_request(
            Request::new(SdkBody::empty()),
            false,
            UnacceptedStage::ReturnedByHyper,
            ConnectorError::user("fresh Hyper failure".into()),
        );
        assert!(fresh_hyper_return.is_err());

        let reused_hyper_return = resolve_unaccepted_request(
            Request::new(SdkBody::empty()),
            true,
            UnacceptedStage::ReturnedByHyper,
            ConnectorError::user("reused failure".into()),
        );
        assert!(matches!(
            reused_hyper_return,
            Ok(H2DispatchOutcome::NotAccepted(_))
        ));
    }
}
