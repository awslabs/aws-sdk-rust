// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::update_custom_action::_update_custom_action_output::UpdateCustomActionOutputBuilder;

pub use crate::operation::update_custom_action::_update_custom_action_input::UpdateCustomActionInputBuilder;

impl crate::operation::update_custom_action::builders::UpdateCustomActionInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::update_custom_action::UpdateCustomActionOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_custom_action::UpdateCustomActionError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.update_custom_action();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UpdateCustomAction`.
///
/// <p>Updates a custom action.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UpdateCustomActionFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::update_custom_action::builders::UpdateCustomActionInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::update_custom_action::UpdateCustomActionOutput,
        crate::operation::update_custom_action::UpdateCustomActionError,
    > for UpdateCustomActionFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::update_custom_action::UpdateCustomActionOutput,
            crate::operation::update_custom_action::UpdateCustomActionError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UpdateCustomActionFluentBuilder {
    /// Creates a new `UpdateCustomActionFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UpdateCustomAction as a reference.
    pub fn as_input(&self) -> &crate::operation::update_custom_action::builders::UpdateCustomActionInputBuilder {
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
        crate::operation::update_custom_action::UpdateCustomActionOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_custom_action::UpdateCustomActionError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::update_custom_action::UpdateCustomAction::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::update_custom_action::UpdateCustomAction::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::update_custom_action::UpdateCustomActionOutput,
        crate::operation::update_custom_action::UpdateCustomActionError,
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
    /// <p>The fully defined Amazon Resource Name (ARN) of the custom action.</p>
    pub fn custom_action_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.custom_action_arn(input.into());
        self
    }
    /// <p>The fully defined Amazon Resource Name (ARN) of the custom action.</p>
    pub fn set_custom_action_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_custom_action_arn(input);
        self
    }
    /// <p>The fully defined Amazon Resource Name (ARN) of the custom action.</p>
    pub fn get_custom_action_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_custom_action_arn()
    }
    /// <p>The definition of the command to run when invoked as an alias or as an action button.</p>
    pub fn definition(mut self, input: crate::types::CustomActionDefinition) -> Self {
        self.inner = self.inner.definition(input);
        self
    }
    /// <p>The definition of the command to run when invoked as an alias or as an action button.</p>
    pub fn set_definition(mut self, input: ::std::option::Option<crate::types::CustomActionDefinition>) -> Self {
        self.inner = self.inner.set_definition(input);
        self
    }
    /// <p>The definition of the command to run when invoked as an alias or as an action button.</p>
    pub fn get_definition(&self) -> &::std::option::Option<crate::types::CustomActionDefinition> {
        self.inner.get_definition()
    }
    /// <p>The name used to invoke this action in the chat channel. For example, <code>@aws run my-alias</code>.</p>
    pub fn alias_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.alias_name(input.into());
        self
    }
    /// <p>The name used to invoke this action in the chat channel. For example, <code>@aws run my-alias</code>.</p>
    pub fn set_alias_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_alias_name(input);
        self
    }
    /// <p>The name used to invoke this action in the chat channel. For example, <code>@aws run my-alias</code>.</p>
    pub fn get_alias_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_alias_name()
    }
    ///
    /// Appends an item to `Attachments`.
    ///
    /// To override the contents of this collection use [`set_attachments`](Self::set_attachments).
    ///
    /// <p>Defines when this custom action button should be attached to a notification.</p>
    pub fn attachments(mut self, input: crate::types::CustomActionAttachment) -> Self {
        self.inner = self.inner.attachments(input);
        self
    }
    /// <p>Defines when this custom action button should be attached to a notification.</p>
    pub fn set_attachments(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::CustomActionAttachment>>) -> Self {
        self.inner = self.inner.set_attachments(input);
        self
    }
    /// <p>Defines when this custom action button should be attached to a notification.</p>
    pub fn get_attachments(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::CustomActionAttachment>> {
        self.inner.get_attachments()
    }
}
