// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::delete_ai_guardrail_version::_delete_ai_guardrail_version_output::DeleteAiGuardrailVersionOutputBuilder;

pub use crate::operation::delete_ai_guardrail_version::_delete_ai_guardrail_version_input::DeleteAiGuardrailVersionInputBuilder;

impl crate::operation::delete_ai_guardrail_version::builders::DeleteAiGuardrailVersionInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::delete_ai_guardrail_version::DeleteAiGuardrailVersionOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::delete_ai_guardrail_version::DeleteAIGuardrailVersionError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.delete_ai_guardrail_version();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `DeleteAIGuardrailVersion`.
///
/// <p>Delete and Amazon Q in Connect AI Guardrail version.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct DeleteAIGuardrailVersionFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::delete_ai_guardrail_version::builders::DeleteAiGuardrailVersionInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::delete_ai_guardrail_version::DeleteAiGuardrailVersionOutput,
        crate::operation::delete_ai_guardrail_version::DeleteAIGuardrailVersionError,
    > for DeleteAIGuardrailVersionFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::delete_ai_guardrail_version::DeleteAiGuardrailVersionOutput,
            crate::operation::delete_ai_guardrail_version::DeleteAIGuardrailVersionError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl DeleteAIGuardrailVersionFluentBuilder {
    /// Creates a new `DeleteAIGuardrailVersionFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the DeleteAIGuardrailVersion as a reference.
    pub fn as_input(&self) -> &crate::operation::delete_ai_guardrail_version::builders::DeleteAiGuardrailVersionInputBuilder {
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
        crate::operation::delete_ai_guardrail_version::DeleteAiGuardrailVersionOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::delete_ai_guardrail_version::DeleteAIGuardrailVersionError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::delete_ai_guardrail_version::DeleteAIGuardrailVersion::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::delete_ai_guardrail_version::DeleteAIGuardrailVersion::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::delete_ai_guardrail_version::DeleteAiGuardrailVersionOutput,
        crate::operation::delete_ai_guardrail_version::DeleteAIGuardrailVersionError,
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
    /// <p>The identifier of the Amazon Q in Connect assistant. Can be either the ID or the ARN. URLs cannot contain the ARN.</p>
    pub fn assistant_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.assistant_id(input.into());
        self
    }
    /// <p>The identifier of the Amazon Q in Connect assistant. Can be either the ID or the ARN. URLs cannot contain the ARN.</p>
    pub fn set_assistant_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_assistant_id(input);
        self
    }
    /// <p>The identifier of the Amazon Q in Connect assistant. Can be either the ID or the ARN. URLs cannot contain the ARN.</p>
    pub fn get_assistant_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_assistant_id()
    }
    /// <p>The identifier of the Amazon Q in Connect AI Guardrail.</p>
    pub fn ai_guardrail_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.ai_guardrail_id(input.into());
        self
    }
    /// <p>The identifier of the Amazon Q in Connect AI Guardrail.</p>
    pub fn set_ai_guardrail_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_ai_guardrail_id(input);
        self
    }
    /// <p>The identifier of the Amazon Q in Connect AI Guardrail.</p>
    pub fn get_ai_guardrail_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_ai_guardrail_id()
    }
    /// <p>The version number of the AI Guardrail version to be deleted.</p>
    pub fn version_number(mut self, input: i64) -> Self {
        self.inner = self.inner.version_number(input);
        self
    }
    /// <p>The version number of the AI Guardrail version to be deleted.</p>
    pub fn set_version_number(mut self, input: ::std::option::Option<i64>) -> Self {
        self.inner = self.inner.set_version_number(input);
        self
    }
    /// <p>The version number of the AI Guardrail version to be deleted.</p>
    pub fn get_version_number(&self) -> &::std::option::Option<i64> {
        self.inner.get_version_number()
    }
}
