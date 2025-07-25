// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::get_run::_get_run_output::GetRunOutputBuilder;

pub use crate::operation::get_run::_get_run_input::GetRunInputBuilder;

impl crate::operation::get_run::builders::GetRunInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::get_run::GetRunOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::get_run::GetRunError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.get_run();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `GetRun`.
///
/// <p>Gets detailed information about a specific run using its ID.</p>
/// <p>Amazon Web Services HealthOmics stores a configurable number of runs, as determined by service limits, that are available to the console and API. If <code>GetRun</code> does not return the requested run, you can find all run logs in the CloudWatch logs. For more information about viewing the run logs, see <a href="https://docs.aws.amazon.com/omics/latest/dev/monitoring-cloudwatch-logs.html">CloudWatch logs</a> in the <i>Amazon Web Services HealthOmics User Guide</i>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct GetRunFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::get_run::builders::GetRunInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl crate::client::customize::internal::CustomizableSend<crate::operation::get_run::GetRunOutput, crate::operation::get_run::GetRunError>
    for GetRunFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<crate::operation::get_run::GetRunOutput, crate::operation::get_run::GetRunError>,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl GetRunFluentBuilder {
    /// Creates a new `GetRunFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the GetRun as a reference.
    pub fn as_input(&self) -> &crate::operation::get_run::builders::GetRunInputBuilder {
        &self.inner
    }
    /// Sends the request and returns the response.
    ///
    /// If an error occurs, an `SdkError` will be returned with additional details that
    /// can be matched against.
    ///
    /// By default, any retryable failures will be retried twice. Retry behavior
    /// is configurable with the [RetryConfig](aws_smithy_types::retry::RetryConfig), which can be
    /// set when configuring the client.
    pub async fn send(
        self,
    ) -> ::std::result::Result<
        crate::operation::get_run::GetRunOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::get_run::GetRunError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::get_run::GetRun::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::get_run::GetRun::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<crate::operation::get_run::GetRunOutput, crate::operation::get_run::GetRunError, Self> {
        crate::client::customize::CustomizableOperation::new(self)
    }
    pub(crate) fn config_override(mut self, config_override: impl ::std::convert::Into<crate::config::Builder>) -> Self {
        self.set_config_override(::std::option::Option::Some(config_override.into()));
        self
    }

    pub(crate) fn set_config_override(&mut self, config_override: ::std::option::Option<crate::config::Builder>) -> &mut Self {
        self.config_override = config_override;
        self
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
