// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::put_restore_validation_result::_put_restore_validation_result_output::PutRestoreValidationResultOutputBuilder;

pub use crate::operation::put_restore_validation_result::_put_restore_validation_result_input::PutRestoreValidationResultInputBuilder;

impl crate::operation::put_restore_validation_result::builders::PutRestoreValidationResultInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::put_restore_validation_result::PutRestoreValidationResultOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::put_restore_validation_result::PutRestoreValidationResultError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.put_restore_validation_result();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `PutRestoreValidationResult`.
///
/// <p>This request allows you to send your independent self-run restore test validation results. <code>RestoreJobId</code> and <code>ValidationStatus</code> are required. Optionally, you can input a <code>ValidationStatusMessage</code>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct PutRestoreValidationResultFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::put_restore_validation_result::builders::PutRestoreValidationResultInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::put_restore_validation_result::PutRestoreValidationResultOutput,
        crate::operation::put_restore_validation_result::PutRestoreValidationResultError,
    > for PutRestoreValidationResultFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::put_restore_validation_result::PutRestoreValidationResultOutput,
            crate::operation::put_restore_validation_result::PutRestoreValidationResultError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl PutRestoreValidationResultFluentBuilder {
    /// Creates a new `PutRestoreValidationResultFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the PutRestoreValidationResult as a reference.
    pub fn as_input(&self) -> &crate::operation::put_restore_validation_result::builders::PutRestoreValidationResultInputBuilder {
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
        crate::operation::put_restore_validation_result::PutRestoreValidationResultOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::put_restore_validation_result::PutRestoreValidationResultError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::put_restore_validation_result::PutRestoreValidationResult::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::put_restore_validation_result::PutRestoreValidationResult::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::put_restore_validation_result::PutRestoreValidationResultOutput,
        crate::operation::put_restore_validation_result::PutRestoreValidationResultError,
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
    /// <p>This is a unique identifier of a restore job within Backup.</p>
    pub fn restore_job_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.restore_job_id(input.into());
        self
    }
    /// <p>This is a unique identifier of a restore job within Backup.</p>
    pub fn set_restore_job_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_restore_job_id(input);
        self
    }
    /// <p>This is a unique identifier of a restore job within Backup.</p>
    pub fn get_restore_job_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_restore_job_id()
    }
    /// <p>The status of your restore validation.</p>
    pub fn validation_status(mut self, input: crate::types::RestoreValidationStatus) -> Self {
        self.inner = self.inner.validation_status(input);
        self
    }
    /// <p>The status of your restore validation.</p>
    pub fn set_validation_status(mut self, input: ::std::option::Option<crate::types::RestoreValidationStatus>) -> Self {
        self.inner = self.inner.set_validation_status(input);
        self
    }
    /// <p>The status of your restore validation.</p>
    pub fn get_validation_status(&self) -> &::std::option::Option<crate::types::RestoreValidationStatus> {
        self.inner.get_validation_status()
    }
    /// <p>This is an optional message string you can input to describe the validation status for the restore test validation.</p>
    pub fn validation_status_message(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.validation_status_message(input.into());
        self
    }
    /// <p>This is an optional message string you can input to describe the validation status for the restore test validation.</p>
    pub fn set_validation_status_message(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_validation_status_message(input);
        self
    }
    /// <p>This is an optional message string you can input to describe the validation status for the restore test validation.</p>
    pub fn get_validation_status_message(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_validation_status_message()
    }
}
