// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

///
/// Fluent builder for the `workflow_version_active` waiter.
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
pub struct WorkflowVersionActiveFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::get_workflow_version::builders::GetWorkflowVersionInputBuilder,
}
impl WorkflowVersionActiveFluentBuilder {
    /// Creates a new `WorkflowVersionActiveFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
        }
    }
    /// Access the GetWorkflowVersion as a reference.
    pub fn as_input(&self) -> &crate::operation::get_workflow_version::builders::GetWorkflowVersionInputBuilder {
        &self.inner
    }
    /// Wait until a workflow version is active.
    pub async fn wait(
        self,
        max_wait: ::std::time::Duration,
    ) -> ::std::result::Result<
        crate::waiters::workflow_version_active::WorkflowVersionActiveFinalPoll,
        crate::waiters::workflow_version_active::WaitUntilWorkflowVersionActiveError,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::waiters::error::WaiterError::construction_failure)?;
        let runtime_plugins = crate::operation::get_workflow_version::GetWorkflowVersion::operation_runtime_plugins(
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
            &crate::operation::get_workflow_version::GetWorkflowVersionOutput,
            &crate::operation::get_workflow_version::GetWorkflowVersionError,
        >| {
            // Matches: {"output":{"path":"status","expected":"ACTIVE","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_workflow_version_a0b9c099115634691(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Success;
            }
            // Matches: {"output":{"path":"status","expected":"CREATING","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_workflow_version_ab538ef2e7cb9d2b4(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Retry;
            }
            // Matches: {"output":{"path":"status","expected":"UPDATING","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_workflow_version_45c85fb7765d90d16(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Retry;
            }
            // Matches: {"output":{"path":"status","expected":"FAILED","comparator":"stringEquals"}}
            if crate::waiters::matchers::match_get_workflow_version_f9c483f08ce8cb218(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Failure;
            }
            ::aws_smithy_runtime::client::waiters::AcceptorState::NoAcceptorsMatched
        };
        let operation = move || {
            let input = input.clone();
            let runtime_plugins = runtime_plugins.clone();
            async move { crate::operation::get_workflow_version::GetWorkflowVersion::orchestrate(&runtime_plugins, input).await }
        };
        let orchestrator = ::aws_smithy_runtime::client::waiters::WaiterOrchestrator::builder()
            .min_delay(::std::time::Duration::from_secs(3))
            .max_delay(::std::time::Duration::from_secs(30))
            .max_wait(max_wait)
            .time_source(time_source)
            .sleep_impl(sleep_impl)
            .acceptor(acceptor)
            .operation(operation)
            .build();
        ::aws_smithy_runtime::client::waiters::attach_waiter_tracing_span(orchestrator.orchestrate()).await
    }
    /// <p>The workflow's ID.</p>
    pub fn workflow_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.workflow_id(input.into());
        self
    }
    /// <p>The workflow's ID.</p>
    pub fn set_workflow_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_workflow_id(input);
        self
    }
    /// <p>The workflow's ID.</p>
    pub fn get_workflow_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_workflow_id()
    }
    /// <p>The workflow version name.</p>
    pub fn version_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.version_name(input.into());
        self
    }
    /// <p>The workflow version name.</p>
    pub fn set_version_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_version_name(input);
        self
    }
    /// <p>The workflow version name.</p>
    pub fn get_version_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_version_name()
    }
    /// <p>The workflow's type.</p>
    pub fn r#type(mut self, input: crate::types::WorkflowType) -> Self {
        self.inner = self.inner.r#type(input);
        self
    }
    /// <p>The workflow's type.</p>
    pub fn set_type(mut self, input: ::std::option::Option<crate::types::WorkflowType>) -> Self {
        self.inner = self.inner.set_type(input);
        self
    }
    /// <p>The workflow's type.</p>
    pub fn get_type(&self) -> &::std::option::Option<crate::types::WorkflowType> {
        self.inner.get_type()
    }
    ///
    /// Appends an item to `export`.
    ///
    /// To override the contents of this collection use [`set_export`](Self::set_export).
    ///
    /// <p>The export format for the workflow.</p>
    pub fn export(mut self, input: crate::types::WorkflowExport) -> Self {
        self.inner = self.inner.export(input);
        self
    }
    /// <p>The export format for the workflow.</p>
    pub fn set_export(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::WorkflowExport>>) -> Self {
        self.inner = self.inner.set_export(input);
        self
    }
    /// <p>The export format for the workflow.</p>
    pub fn get_export(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::WorkflowExport>> {
        self.inner.get_export()
    }
    /// <p>Amazon Web Services Id of the owner of the workflow.</p>
    pub fn workflow_owner_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.workflow_owner_id(input.into());
        self
    }
    /// <p>Amazon Web Services Id of the owner of the workflow.</p>
    pub fn set_workflow_owner_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_workflow_owner_id(input);
        self
    }
    /// <p>Amazon Web Services Id of the owner of the workflow.</p>
    pub fn get_workflow_owner_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_workflow_owner_id()
    }
}

/// Successful return type for the `workflow_version_active` waiter.
pub type WorkflowVersionActiveFinalPoll = ::aws_smithy_runtime_api::client::waiters::FinalPoll<
    crate::operation::get_workflow_version::GetWorkflowVersionOutput,
    ::aws_smithy_runtime_api::client::result::SdkError<
        crate::operation::get_workflow_version::GetWorkflowVersionError,
        ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    >,
>;

/// Error type for the `workflow_version_active` waiter.
pub type WaitUntilWorkflowVersionActiveError = ::aws_smithy_runtime_api::client::waiters::error::WaiterError<
    crate::operation::get_workflow_version::GetWorkflowVersionOutput,
    crate::operation::get_workflow_version::GetWorkflowVersionError,
>;
