// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

///
/// Fluent builder for the `service_pipeline_deployed` waiter.
///
/// This builder is intended to be used similar to the other fluent builders for
/// normal operations on the client. However, instead of a `send` method, it has
/// a `wait` method that takes a maximum amount of time to wait.
///
/// Construct this fluent builder using the client by importing the
/// [`Waiters`](crate::client::Waiters) trait and calling the methods
/// prefixed with `wait_until`.
///
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct ServicePipelineDeployedFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::get_service::builders::GetServiceInputBuilder,
}
impl ServicePipelineDeployedFluentBuilder {
    /// Creates a new `ServicePipelineDeployedFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
        }
    }
    /// Access the GetService as a reference.
    pub fn as_input(&self) -> &crate::operation::get_service::builders::GetServiceInputBuilder {
        &self.inner
    }
    /// Wait until an ServicePipeline is deployed. Use this after invoking CreateService or UpdateServicePipeline
    pub async fn wait(
        self,
        max_wait: ::std::time::Duration,
    ) -> ::std::result::Result<
        crate::waiters::service_pipeline_deployed::ServicePipelineDeployedFinalPoll,
        crate::waiters::service_pipeline_deployed::WaitUntilServicePipelineDeployedError,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::waiters::error::WaiterError::construction_failure)?;
        let runtime_plugins = crate::operation::get_service::GetService::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            ::std::option::Option::None,
        )
        .with_operation_plugin(crate::sdk_feature_tracker::waiter::WaiterFeatureTrackerRuntimePlugin::new());
        let mut cfg = ::aws_smithy_types::config_bag::ConfigBag::base();
        let runtime_components_builder = runtime_plugins
            .apply_client_configuration(&mut cfg)
            .map_err(::aws_smithy_runtime_api::client::waiters::error::WaiterError::construction_failure)?;
        let time_components = runtime_components_builder.into_time_components();
        let sleep_impl = time_components.sleep_impl().expect("a sleep impl is required by waiters");
        let time_source = time_components.time_source().expect("a time source is required by waiters");

        let acceptor = move |result: ::std::result::Result<
            &crate::operation::get_service::GetServiceOutput,
            &crate::operation::get_service::GetServiceError,
        >| {
            // Matches: {"output":{"path":"service.pipeline.deploymentStatus","expected":"SUCCEEDED","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_service_be7a287507c1de454(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Success;
            }
            // Matches: {"output":{"path":"service.pipeline.deploymentStatus","expected":"FAILED","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_service_f2dcb04fda03e8ed1(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Failure;
            }
            ::aws_smithy_runtime::client::waiters::AcceptorState::NoAcceptorsMatched
        };
        let operation = move || {
            let input = input.clone();
            let runtime_plugins = runtime_plugins.clone();
            async move { crate::operation::get_service::GetService::orchestrate(&runtime_plugins, input).await }
        };
        let orchestrator = ::aws_smithy_runtime::client::waiters::WaiterOrchestrator::builder()
            .min_delay(::std::time::Duration::from_secs(10))
            .max_delay(::std::time::Duration::from_secs(3600))
            .max_wait(max_wait)
            .time_source(time_source)
            .sleep_impl(sleep_impl)
            .acceptor(acceptor)
            .operation(operation)
            .build();
        ::aws_smithy_runtime::client::waiters::attach_waiter_tracing_span(orchestrator.orchestrate()).await
    }
    /// <p>The name of the service that you want to get the detailed data for.</p>
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.name(input.into());
        self
    }
    /// <p>The name of the service that you want to get the detailed data for.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_name(input);
        self
    }
    /// <p>The name of the service that you want to get the detailed data for.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_name()
    }
}

/// Successful return type for the `service_pipeline_deployed` waiter.
pub type ServicePipelineDeployedFinalPoll = ::aws_smithy_runtime_api::client::waiters::FinalPoll<
    crate::operation::get_service::GetServiceOutput,
    ::aws_smithy_runtime_api::client::result::SdkError<
        crate::operation::get_service::GetServiceError,
        ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    >,
>;

/// Error type for the `service_pipeline_deployed` waiter.
pub type WaitUntilServicePipelineDeployedError = ::aws_smithy_runtime_api::client::waiters::error::WaiterError<
    crate::operation::get_service::GetServiceOutput,
    crate::operation::get_service::GetServiceError,
>;
