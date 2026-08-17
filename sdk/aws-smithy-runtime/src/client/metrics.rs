/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_async::time::{SharedTimeSource, TimeSource};
use aws_smithy_observability::{
    global::get_telemetry_provider, instruments::Histogram, AttributeValue, Attributes,
    ObservabilityError,
};
use aws_smithy_runtime_api::client::{
    interceptors::{dyn_dispatch_hint, Intercept, SharedInterceptor},
    orchestrator::Metadata,
    runtime_components::RuntimeComponentsBuilder,
    runtime_plugin::RuntimePlugin,
};
use aws_smithy_types::config_bag::{FrozenLayer, Layer, Storable, StoreReplace};
use aws_smithy_types::telemetry::{CapturedTelemetryAttributes, RequestedTelemetryAttributes};
use std::{borrow::Cow, sync::Arc, time::SystemTime};

/// Sets the outcome attributes (`error.type` and `http.status_code`) on `attrs` from a
/// finalizer-phase context.
fn add_outcome_attrs(
    attrs: &mut Attributes,
    context: &aws_smithy_runtime_api::client::interceptors::context::FinalizerInterceptorContextRef<
        '_,
    >,
) {
    // Coarse category only; the error is type-erased here, so the modeled name isn't reachable.
    // Absent on success, per OTel convention.
    if let Some(Err(err)) = context.output_or_error() {
        let category = if err.is_timeout_error() {
            "timeout"
        } else if err.is_connector_error() {
            "connector"
        } else if err.is_response_error() {
            "response"
        } else if err.is_operation_error() {
            "operation"
        } else {
            "other"
        };
        attrs.set("error.type", AttributeValue::String(category.into()));
    }

    // Raw HTTP status code, whenever a response reached us.
    if let Some(response) = context.response() {
        attrs.set(
            "http.status_code",
            AttributeValue::I64(i64::from(response.status().as_u16())),
        );
    }
}

/// Struct to hold metric data in the ConfigBag
#[derive(Debug, Clone)]
pub(crate) struct MeasurementsContainer {
    call_start: SystemTime,
    attempts: u32,
    attempt_start: SystemTime,
}

impl Storable for MeasurementsContainer {
    type Storer = StoreReplace<Self>;
}

/// Instruments for recording a single operation
#[derive(Debug, Clone)]
pub(crate) struct OperationTelemetry {
    pub(crate) operation_duration: Arc<dyn Histogram>,
    pub(crate) attempt_duration: Arc<dyn Histogram>,
    // Body sizes are their own instruments rather than attributes on the duration histogram: body
    // size is near-unique per call, so attaching it as a label would fragment the duration metric
    // into one time series per byte count.
    pub(crate) request_body_size: Arc<dyn Histogram>,
    pub(crate) response_body_size: Arc<dyn Histogram>,
}

impl OperationTelemetry {
    pub(crate) fn new(scope: &'static str) -> Result<Self, ObservabilityError> {
        let meter = get_telemetry_provider()?
            .meter_provider()
            .get_meter(scope, None);

        Ok(Self{
            operation_duration: meter
                .create_histogram("smithy.client.call.duration")
                .set_units("s")
                .set_description("Overall call duration (including retries and time to send or receive request and response body)")
                .build(),
            attempt_duration: meter
                .create_histogram("smithy.client.call.attempt.duration")
                .set_units("s")
                .set_description("The time it takes to connect to the service, send the request, and get back HTTP status code and headers (including time queued waiting to be sent)")
                .build(),
            request_body_size: meter
                .create_histogram("smithy.client.call.request.size")
                .set_units("By")
                .set_description("Size of the transferred request body, in bytes")
                .build(),
            response_body_size: meter
                .create_histogram("smithy.client.call.response.size")
                .set_units("By")
                .set_description("Size of the transferred response body, in bytes")
                .build(),
        })
    }
}

impl Storable for OperationTelemetry {
    type Storer = StoreReplace<Self>;
}

#[derive(Debug)]
pub(crate) struct MetricsInterceptor {
    // Holding a TimeSource here isn't ideal, but RuntimeComponents aren't available in
    // the read_before_execution hook and that is when we need to start the timer for
    // the operation.
    time_source: SharedTimeSource,
}

impl MetricsInterceptor {
    pub(crate) fn new(time_source: SharedTimeSource) -> Result<Self, ObservabilityError> {
        Ok(MetricsInterceptor { time_source })
    }

    pub(crate) fn get_attrs_from_cfg(
        &self,
        cfg: &aws_smithy_types::config_bag::ConfigBag,
    ) -> Option<Attributes> {
        let operation_metadata = cfg.load::<Metadata>();

        if let Some(md) = operation_metadata {
            let mut attributes = Attributes::new();
            attributes.set("rpc.service", AttributeValue::String(md.service().into()));
            attributes.set("rpc.method", AttributeValue::String(md.name().into()));

            // Merge captured input members that the customer opted in to *emit*. Capture-only
            // members are present in the bag for in-process reads but are deliberately excluded
            // from the metric label set.
            if let (Some(captured), Some(requested)) = (
                cfg.load::<CapturedTelemetryAttributes>(),
                cfg.load::<RequestedTelemetryAttributes>(),
            ) {
                for (name, value) in captured.iter() {
                    if requested.should_emit(name) {
                        attributes.set(name, AttributeValue::String(value.into()));
                    }
                }
            }

            Some(attributes)
        } else {
            None
        }
    }

    pub(crate) fn get_measurements_and_instruments<'a>(
        &self,
        cfg: &'a aws_smithy_types::config_bag::ConfigBag,
    ) -> (&'a MeasurementsContainer, &'a OperationTelemetry) {
        let measurements = cfg
            .load::<MeasurementsContainer>()
            .expect("set in `read_before_execution`");

        let instruments = cfg
            .load::<OperationTelemetry>()
            .expect("set in RuntimePlugin");

        (measurements, instruments)
    }
}

#[dyn_dispatch_hint]
impl Intercept for MetricsInterceptor {
    fn name(&self) -> &'static str {
        "MetricsInterceptor"
    }

    fn read_before_execution(
        &self,
        _context: &aws_smithy_runtime_api::client::interceptors::context::BeforeSerializationInterceptorContextRef<'_>,
        cfg: &mut aws_smithy_types::config_bag::ConfigBag,
    ) -> Result<(), aws_smithy_runtime_api::box_error::BoxError> {
        cfg.interceptor_state().store_put(MeasurementsContainer {
            call_start: self.time_source.now(),
            attempts: 0,
            attempt_start: SystemTime::UNIX_EPOCH,
        });

        Ok(())
    }

    fn read_after_execution(
        &self,
        context: &aws_smithy_runtime_api::client::interceptors::context::FinalizerInterceptorContextRef<'_>,
        _runtime_components: &aws_smithy_runtime_api::client::runtime_components::RuntimeComponents,
        cfg: &mut aws_smithy_types::config_bag::ConfigBag,
    ) -> Result<(), aws_smithy_runtime_api::box_error::BoxError> {
        let (measurements, instruments) = self.get_measurements_and_instruments(cfg);

        let attributes = self.get_attrs_from_cfg(cfg);

        if let Some(mut attrs) = attributes {
            // The outcome is only known at the finalizer, so it is set here rather than in
            // `get_attrs_from_cfg` (which also serves the per-attempt path).
            add_outcome_attrs(&mut attrs, context);

            // Transferred byte sizes are recorded on their own instruments by the byte
            // interceptor (see `telemetry_bytes`), not as attributes on the duration histogram.

            let call_end = self.time_source.now();
            let call_duration = call_end.duration_since(measurements.call_start);
            if let Ok(elapsed) = call_duration {
                instruments
                    .operation_duration
                    .record(elapsed.as_secs_f64(), Some(&attrs), None);
            }
        }

        Ok(())
    }

    fn read_before_attempt(
        &self,
        _context: &aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextRef<'_>,
        _runtime_components: &aws_smithy_runtime_api::client::runtime_components::RuntimeComponents,
        cfg: &mut aws_smithy_types::config_bag::ConfigBag,
    ) -> Result<(), aws_smithy_runtime_api::box_error::BoxError> {
        let measurements = cfg
            .get_mut::<MeasurementsContainer>()
            .expect("set in `read_before_execution`");

        measurements.attempts += 1;
        measurements.attempt_start = self.time_source.now();

        Ok(())
    }

    fn read_after_attempt(
        &self,
        _context: &aws_smithy_runtime_api::client::interceptors::context::FinalizerInterceptorContextRef<'_>,
        _runtime_components: &aws_smithy_runtime_api::client::runtime_components::RuntimeComponents,
        cfg: &mut aws_smithy_types::config_bag::ConfigBag,
    ) -> Result<(), aws_smithy_runtime_api::box_error::BoxError> {
        let (measurements, instruments) = self.get_measurements_and_instruments(cfg);

        let attempt_end = self.time_source.now();
        let attempt_duration = attempt_end.duration_since(measurements.attempt_start);
        let attributes = self.get_attrs_from_cfg(cfg);

        if let (Ok(elapsed), Some(mut attrs)) = (attempt_duration, attributes) {
            attrs.set("attempt", AttributeValue::I64(measurements.attempts.into()));

            instruments
                .attempt_duration
                .record(elapsed.as_secs_f64(), Some(&attrs), None);
        }
        Ok(())
    }
}

/// Runtime plugin that adds an interceptor for collecting metrics
#[derive(Debug, Default)]
pub struct MetricsRuntimePlugin {
    scope: &'static str,
    time_source: SharedTimeSource,
    metadata: Option<Metadata>,
}

impl MetricsRuntimePlugin {
    /// Create a [MetricsRuntimePluginBuilder]
    pub fn builder() -> MetricsRuntimePluginBuilder {
        MetricsRuntimePluginBuilder::default()
    }
}

impl RuntimePlugin for MetricsRuntimePlugin {
    fn runtime_components(
        &self,
        _current_components: &RuntimeComponentsBuilder,
    ) -> Cow<'_, RuntimeComponentsBuilder> {
        let interceptor = MetricsInterceptor::new(self.time_source.clone());
        if let Ok(interceptor) = interceptor {
            Cow::Owned(
                RuntimeComponentsBuilder::new("Metrics")
                    .with_interceptor(SharedInterceptor::permanent(interceptor))
                    // Counts transferred bytes into the bag for the metrics interceptor to read.
                    .with_interceptor(SharedInterceptor::permanent(
                        crate::client::telemetry_bytes::TelemetryBytesInterceptor,
                    )),
            )
        } else {
            Cow::Owned(RuntimeComponentsBuilder::new("Metrics"))
        }
    }

    fn config(&self) -> Option<FrozenLayer> {
        let instruments = OperationTelemetry::new(self.scope);

        if let Ok(instruments) = instruments {
            let mut cfg = Layer::new("Metrics");
            cfg.store_put(instruments);

            if let Some(metadata) = &self.metadata {
                cfg.store_put(metadata.clone());
            }

            Some(cfg.freeze())
        } else {
            None
        }
    }
}

/// Builder for [MetricsRuntimePlugin]
#[derive(Debug, Default)]
pub struct MetricsRuntimePluginBuilder {
    scope: Option<&'static str>,
    time_source: Option<SharedTimeSource>,
    metadata: Option<Metadata>,
}

impl MetricsRuntimePluginBuilder {
    /// Set the scope for the metrics
    pub fn with_scope(mut self, scope: &'static str) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Set the [TimeSource] for the metrics
    pub fn with_time_source(mut self, time_source: impl TimeSource + 'static) -> Self {
        self.time_source = Some(SharedTimeSource::new(time_source));
        self
    }

    /// Set the [Metadata] for the metrics.
    ///
    /// Note: the Metadata is optional, most operations set it themselves, but this is useful
    /// for operations that do not, like some of the credential providers.
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Build a [MetricsRuntimePlugin]
    pub fn build(
        self,
    ) -> Result<MetricsRuntimePlugin, aws_smithy_runtime_api::box_error::BoxError> {
        if let Some(scope) = self.scope {
            Ok(MetricsRuntimePlugin {
                scope,
                time_source: self.time_source.unwrap_or_default(),
                metadata: self.metadata,
            })
        } else {
            Err("Scope is required for MetricsRuntimePlugin.".into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use aws_smithy_async::time::SystemTimeSource;
    use aws_smithy_types::config_bag::ConfigBag;

    fn interceptor() -> MetricsInterceptor {
        MetricsInterceptor::new(SharedTimeSource::new(SystemTimeSource::new())).unwrap()
    }

    fn cfg_with(layer: Layer) -> ConfigBag {
        ConfigBag::of_layers(vec![layer])
    }

    fn string_attr<'a>(attrs: &'a Attributes, key: &str) -> Option<&'a str> {
        match attrs.get(key) {
            Some(AttributeValue::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    #[test]
    fn base_attrs_are_service_and_method() {
        let mut layer = Layer::new("test");
        layer.store_put(Metadata::new("GetObject", "S3"));

        let attrs = interceptor()
            .get_attrs_from_cfg(&cfg_with(layer))
            .expect("metadata present");

        assert_eq!(Some("S3"), string_attr(&attrs, "rpc.service"));
        assert_eq!(Some("GetObject"), string_attr(&attrs, "rpc.method"));
    }

    #[test]
    fn no_attrs_without_metadata() {
        // Nothing to key the metric on, so no attributes are produced.
        assert!(interceptor()
            .get_attrs_from_cfg(&cfg_with(Layer::new("test")))
            .is_none());
    }

    #[test]
    fn emitted_members_are_merged_onto_attrs() {
        let mut captured = CapturedTelemetryAttributes::new();
        captured.insert("Bucket", "my-bucket");

        let mut layer = Layer::new("test");
        layer.store_put(Metadata::new("GetObject", "S3"));
        layer.store_put(captured);
        layer.store_put(RequestedTelemetryAttributes::new(["Bucket"]));

        let attrs = interceptor()
            .get_attrs_from_cfg(&cfg_with(layer))
            .expect("metadata present");

        // The emitted input member rides alongside the built-in rpc.* attributes.
        assert_eq!(Some("my-bucket"), string_attr(&attrs, "Bucket"));
        assert_eq!(Some("S3"), string_attr(&attrs, "rpc.service"));
    }

    #[test]
    fn capture_only_members_are_not_emitted() {
        // A value captured for in-process reads must not land on the metric.
        let mut captured = CapturedTelemetryAttributes::new();
        captured.insert("Prefix", "logs/");

        let mut requested = RequestedTelemetryAttributes::default();
        requested.capture_only(["Prefix"]);

        let mut layer = Layer::new("test");
        layer.store_put(Metadata::new("GetObject", "S3"));
        layer.store_put(captured);
        layer.store_put(requested);

        let attrs = interceptor()
            .get_attrs_from_cfg(&cfg_with(layer))
            .expect("metadata present");

        assert!(
            attrs.get("Prefix").is_none(),
            "capture-only member must not be emitted on the metric"
        );
    }

    #[test]
    fn nothing_captured_leaves_only_base_attrs() {
        // Opt-in is off by default: an empty capture set adds nothing.
        let mut layer = Layer::new("test");
        layer.store_put(Metadata::new("GetObject", "S3"));
        layer.store_put(CapturedTelemetryAttributes::new());

        let attrs = interceptor()
            .get_attrs_from_cfg(&cfg_with(layer))
            .expect("metadata present");

        assert_eq!(Some("GetObject"), string_attr(&attrs, "rpc.method"));
        assert!(attrs.get("Bucket").is_none());
    }

    // --- add_outcome_attrs (the `status` dimension) ---

    use aws_smithy_runtime_api::client::interceptors::context::{
        Error, Input, InterceptorContext, Output,
    };
    use aws_smithy_runtime_api::client::orchestrator::OrchestratorError;
    use aws_smithy_runtime_api::client::result::ConnectorError;
    use aws_smithy_runtime_api::http::{Response, StatusCode};
    use aws_smithy_types::body::SdkBody;

    fn i64_attr(attrs: &Attributes, key: &str) -> Option<i64> {
        match attrs.get(key) {
            Some(AttributeValue::I64(v)) => Some(*v),
            _ => None,
        }
    }

    #[test]
    fn outcome_on_success_has_status_code_and_no_error_type() {
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_output_or_error(Ok(Output::doesnt_matter()));
        ctx.set_response(Response::new(
            StatusCode::try_from(200).unwrap(),
            SdkBody::empty(),
        ));

        let mut attrs = Attributes::new();
        add_outcome_attrs(&mut attrs, &(&ctx).into());

        // error.type is absent on success (OTel convention); status code is present.
        assert!(attrs.get("error.type").is_none());
        assert_eq!(Some(200), i64_attr(&attrs, "http.status_code"));
    }

    #[test]
    fn outcome_on_failure_sets_error_type_category() {
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_output_or_error(Err(OrchestratorError::connector(ConnectorError::io(
            "boom".into(),
        ))));

        let mut attrs = Attributes::new();
        add_outcome_attrs(&mut attrs, &(&ctx).into());

        // A connector error maps to the `connector` category.
        assert_eq!(Some("connector"), string_attr(&attrs, "error.type"));
    }

    #[test]
    fn outcome_without_response_omits_status_code() {
        let mut ctx: InterceptorContext<Input, Output, Error> =
            InterceptorContext::new(Input::doesnt_matter());
        ctx.set_output_or_error(Err(OrchestratorError::connector(ConnectorError::io(
            "boom".into(),
        ))));

        let mut attrs = Attributes::new();
        add_outcome_attrs(&mut attrs, &(&ctx).into());

        // No response reached us, so there is no HTTP status code to record.
        assert!(attrs.get("http.status_code").is_none());
        assert_eq!(Some("connector"), string_attr(&attrs, "error.type"));
    }
}
