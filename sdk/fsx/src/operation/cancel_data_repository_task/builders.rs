// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::cancel_data_repository_task::_cancel_data_repository_task_output::CancelDataRepositoryTaskOutputBuilder;

pub use crate::operation::cancel_data_repository_task::_cancel_data_repository_task_input::CancelDataRepositoryTaskInputBuilder;

impl crate::operation::cancel_data_repository_task::builders::CancelDataRepositoryTaskInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::cancel_data_repository_task::CancelDataRepositoryTaskOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::cancel_data_repository_task::CancelDataRepositoryTaskError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.cancel_data_repository_task();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CancelDataRepositoryTask`.
///
/// <p>Cancels an existing Amazon FSx for Lustre data repository task if that task is in either the <code>PENDING</code> or <code>EXECUTING</code> state. When you cancel an export task, Amazon FSx does the following.</p>
/// <ul>
/// <li>
/// <p>Any files that FSx has already exported are not reverted.</p></li>
/// <li>
/// <p>FSx continues to export any files that are in-flight when the cancel operation is received.</p></li>
/// <li>
/// <p>FSx does not export any files that have not yet been exported.</p></li>
/// </ul>
/// <p>For a release task, Amazon FSx will stop releasing files upon cancellation. Any files that have already been released will remain in the released state.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CancelDataRepositoryTaskFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::cancel_data_repository_task::builders::CancelDataRepositoryTaskInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::cancel_data_repository_task::CancelDataRepositoryTaskOutput,
        crate::operation::cancel_data_repository_task::CancelDataRepositoryTaskError,
    > for CancelDataRepositoryTaskFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::cancel_data_repository_task::CancelDataRepositoryTaskOutput,
            crate::operation::cancel_data_repository_task::CancelDataRepositoryTaskError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CancelDataRepositoryTaskFluentBuilder {
    /// Creates a new `CancelDataRepositoryTaskFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CancelDataRepositoryTask as a reference.
    pub fn as_input(&self) -> &crate::operation::cancel_data_repository_task::builders::CancelDataRepositoryTaskInputBuilder {
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
        crate::operation::cancel_data_repository_task::CancelDataRepositoryTaskOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::cancel_data_repository_task::CancelDataRepositoryTaskError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::cancel_data_repository_task::CancelDataRepositoryTask::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::cancel_data_repository_task::CancelDataRepositoryTask::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::cancel_data_repository_task::CancelDataRepositoryTaskOutput,
        crate::operation::cancel_data_repository_task::CancelDataRepositoryTaskError,
        Self,
    > {
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
    /// <p>Specifies the data repository task to cancel.</p>
    pub fn task_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.task_id(input.into());
        self
    }
    /// <p>Specifies the data repository task to cancel.</p>
    pub fn set_task_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_task_id(input);
        self
    }
    /// <p>Specifies the data repository task to cancel.</p>
    pub fn get_task_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_task_id()
    }
}
