// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::delete_message_template_attachment::_delete_message_template_attachment_output::DeleteMessageTemplateAttachmentOutputBuilder;

pub use crate::operation::delete_message_template_attachment::_delete_message_template_attachment_input::DeleteMessageTemplateAttachmentInputBuilder;

impl crate::operation::delete_message_template_attachment::builders::DeleteMessageTemplateAttachmentInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachmentOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachmentError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.delete_message_template_attachment();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `DeleteMessageTemplateAttachment`.
///
/// <p>Deletes the attachment file from the Amazon Q in Connect message template that is referenced by <code>$LATEST</code> qualifier. Attachments on available message template versions will remain unchanged.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct DeleteMessageTemplateAttachmentFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::delete_message_template_attachment::builders::DeleteMessageTemplateAttachmentInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachmentOutput,
        crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachmentError,
    > for DeleteMessageTemplateAttachmentFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachmentOutput,
            crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachmentError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl DeleteMessageTemplateAttachmentFluentBuilder {
    /// Creates a new `DeleteMessageTemplateAttachmentFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the DeleteMessageTemplateAttachment as a reference.
    pub fn as_input(&self) -> &crate::operation::delete_message_template_attachment::builders::DeleteMessageTemplateAttachmentInputBuilder {
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
        crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachmentOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachmentError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachment::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachment::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachmentOutput,
        crate::operation::delete_message_template_attachment::DeleteMessageTemplateAttachmentError,
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
    /// <p>The identifier of the knowledge base. Can be either the ID or the ARN. URLs cannot contain the ARN.</p>
    pub fn knowledge_base_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.knowledge_base_id(input.into());
        self
    }
    /// <p>The identifier of the knowledge base. Can be either the ID or the ARN. URLs cannot contain the ARN.</p>
    pub fn set_knowledge_base_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_knowledge_base_id(input);
        self
    }
    /// <p>The identifier of the knowledge base. Can be either the ID or the ARN. URLs cannot contain the ARN.</p>
    pub fn get_knowledge_base_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_knowledge_base_id()
    }
    /// <p>The identifier of the message template. Can be either the ID or the ARN. It cannot contain any qualifier.</p>
    pub fn message_template_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.message_template_id(input.into());
        self
    }
    /// <p>The identifier of the message template. Can be either the ID or the ARN. It cannot contain any qualifier.</p>
    pub fn set_message_template_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_message_template_id(input);
        self
    }
    /// <p>The identifier of the message template. Can be either the ID or the ARN. It cannot contain any qualifier.</p>
    pub fn get_message_template_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_message_template_id()
    }
    /// <p>The identifier of the attachment file.</p>
    pub fn attachment_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.attachment_id(input.into());
        self
    }
    /// <p>The identifier of the attachment file.</p>
    pub fn set_attachment_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_attachment_id(input);
        self
    }
    /// <p>The identifier of the attachment file.</p>
    pub fn get_attachment_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_attachment_id()
    }
}
