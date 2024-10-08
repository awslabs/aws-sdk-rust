// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

///
/// Fluent builder for the `run_completed` waiter.
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
pub struct RunCompletedFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::get_run::builders::GetRunInputBuilder,
}
impl RunCompletedFluentBuilder {
    /// Creates a new `RunCompletedFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
        }
    }
    /// Access the GetRun as a reference.
    pub fn as_input(&self) -> &crate::operation::get_run::builders::GetRunInputBuilder {
        &self.inner
    }
    /// Wait until a run is completed.
    pub async fn wait(
        self,
        max_wait: ::std::time::Duration,
    ) -> ::std::result::Result<crate::waiters::run_completed::RunCompletedFinalPoll, crate::waiters::run_completed::WaitUntilRunCompletedError> {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::waiters::error::WaiterError::construction_failure)?;
        let runtime_plugins = crate::operation::get_run::GetRun::operation_runtime_plugins(
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

        let acceptor = move |result: ::std::result::Result<&crate::operation::get_run::GetRunOutput, &crate::operation::get_run::GetRunError>| {
            // Matches: {"output":{"path":"status","expected":"COMPLETED","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_run_861efe6eafdf84171(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Success;
            }
            // Matches: {"output":{"path":"status","expected":"PENDING","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_run_cf7012a6288344f38(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Retry;
            }
            // Matches: {"output":{"path":"status","expected":"STARTING","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_run_52681d39a0a9cabe6(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Retry;
            }
            // Matches: {"output":{"path":"status","expected":"RUNNING","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_run_7943b3b995e719e2c(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Retry;
            }
            // Matches: {"output":{"path":"status","expected":"STOPPING","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_run_824819fe20f7c07ec(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Retry;
            }
            // Matches: {"output":{"path":"status","expected":"FAILED","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_run_f9c483f08ce8cb218(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Failure;
            }
            ::aws_smithy_runtime::client::waiters::AcceptorState::NoAcceptorsMatched
        };
        let operation = move || {
            let input = input.clone();
            let runtime_plugins = runtime_plugins.clone();
            async move { crate::operation::get_run::GetRun::orchestrate(&runtime_plugins, input).await }
        };
        let orchestrator = ::aws_smithy_runtime::client::waiters::WaiterOrchestrator::builder()
            .min_delay(::std::time::Duration::from_secs(30))
            .max_delay(::std::time::Duration::from_secs(600))
            .max_wait(max_wait)
            .time_source(time_source)
            .sleep_impl(sleep_impl)
            .acceptor(acceptor)
            .operation(operation)
            .build();
        ::aws_smithy_runtime::client::waiters::attach_waiter_tracing_span(orchestrator.orchestrate()).await
    }
    /// <p>The run's ID.</p>
    pub fn id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.id(input.into());
        self
    }
    /// <p>The run's ID.</p>
    pub fn set_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_id(input);
        self
    }
    /// <p>The run's ID.</p>
    pub fn get_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_id()
    }
    ///
    /// Appends an item to `export`.
    ///
    /// To override the contents of this collection use [`set_export`](Self::set_export).
    ///
    /// <p>The run's export format.</p>
    pub fn export(mut self, input: crate::types::RunExport) -> Self {
        self.inner = self.inner.export(input);
        self
    }
    /// <p>The run's export format.</p>
    pub fn set_export(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::RunExport>>) -> Self {
        self.inner = self.inner.set_export(input);
        self
    }
    /// <p>The run's export format.</p>
    pub fn get_export(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::RunExport>> {
        self.inner.get_export()
    }
}

/// Successful return type for the `run_completed` waiter.
pub type RunCompletedFinalPoll = ::aws_smithy_runtime_api::client::waiters::FinalPoll<
    crate::operation::get_run::GetRunOutput,
    ::aws_smithy_runtime_api::client::result::SdkError<
        crate::operation::get_run::GetRunError,
        ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    >,
>;

/// Error type for the `run_completed` waiter.
pub type WaitUntilRunCompletedError =
    ::aws_smithy_runtime_api::client::waiters::error::WaiterError<crate::operation::get_run::GetRunOutput, crate::operation::get_run::GetRunError>;
