// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::import_document::_import_document_output::ImportDocumentOutputBuilder;

pub use crate::operation::import_document::_import_document_input::ImportDocumentInputBuilder;

impl crate::operation::import_document::builders::ImportDocumentInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::import_document::ImportDocumentOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::import_document::ImportDocumentError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.import_document();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `ImportDocument`.
///
/// <p>Uploads a file that can then be used either as a default in a <code>FileUploadCard</code> from Q App definition or as a file that is used inside a single Q App run. The purpose of the document is determined by a scope parameter that indicates whether it is at the app definition level or at the app session level.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct ImportDocumentFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::import_document::builders::ImportDocumentInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::import_document::ImportDocumentOutput,
        crate::operation::import_document::ImportDocumentError,
    > for ImportDocumentFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::import_document::ImportDocumentOutput,
            crate::operation::import_document::ImportDocumentError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl ImportDocumentFluentBuilder {
    /// Creates a new `ImportDocumentFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the ImportDocument as a reference.
    pub fn as_input(&self) -> &crate::operation::import_document::builders::ImportDocumentInputBuilder {
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
        crate::operation::import_document::ImportDocumentOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::import_document::ImportDocumentError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::import_document::ImportDocument::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::import_document::ImportDocument::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::import_document::ImportDocumentOutput,
        crate::operation::import_document::ImportDocumentError,
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
    /// <p>The unique identifier of the Amazon Q Business application environment instance.</p>
    pub fn instance_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.instance_id(input.into());
        self
    }
    /// <p>The unique identifier of the Amazon Q Business application environment instance.</p>
    pub fn set_instance_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_instance_id(input);
        self
    }
    /// <p>The unique identifier of the Amazon Q Business application environment instance.</p>
    pub fn get_instance_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_instance_id()
    }
    /// <p>The unique identifier of the card the file is associated with.</p>
    pub fn card_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.card_id(input.into());
        self
    }
    /// <p>The unique identifier of the card the file is associated with.</p>
    pub fn set_card_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_card_id(input);
        self
    }
    /// <p>The unique identifier of the card the file is associated with.</p>
    pub fn get_card_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_card_id()
    }
    /// <p>The unique identifier of the Q App the file is associated with.</p>
    pub fn app_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.app_id(input.into());
        self
    }
    /// <p>The unique identifier of the Q App the file is associated with.</p>
    pub fn set_app_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_app_id(input);
        self
    }
    /// <p>The unique identifier of the Q App the file is associated with.</p>
    pub fn get_app_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_app_id()
    }
    /// <p>The base64-encoded contents of the file to upload.</p>
    pub fn file_contents_base64(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.file_contents_base64(input.into());
        self
    }
    /// <p>The base64-encoded contents of the file to upload.</p>
    pub fn set_file_contents_base64(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_file_contents_base64(input);
        self
    }
    /// <p>The base64-encoded contents of the file to upload.</p>
    pub fn get_file_contents_base64(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_file_contents_base64()
    }
    /// <p>The name of the file being uploaded.</p>
    pub fn file_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.file_name(input.into());
        self
    }
    /// <p>The name of the file being uploaded.</p>
    pub fn set_file_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_file_name(input);
        self
    }
    /// <p>The name of the file being uploaded.</p>
    pub fn get_file_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_file_name()
    }
    /// <p>Whether the file is associated with a Q App definition or a specific Q App session.</p>
    pub fn scope(mut self, input: crate::types::DocumentScope) -> Self {
        self.inner = self.inner.scope(input);
        self
    }
    /// <p>Whether the file is associated with a Q App definition or a specific Q App session.</p>
    pub fn set_scope(mut self, input: ::std::option::Option<crate::types::DocumentScope>) -> Self {
        self.inner = self.inner.set_scope(input);
        self
    }
    /// <p>Whether the file is associated with a Q App definition or a specific Q App session.</p>
    pub fn get_scope(&self) -> &::std::option::Option<crate::types::DocumentScope> {
        self.inner.get_scope()
    }
    /// <p>The unique identifier of the Q App session the file is associated with, if applicable.</p>
    pub fn session_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.session_id(input.into());
        self
    }
    /// <p>The unique identifier of the Q App session the file is associated with, if applicable.</p>
    pub fn set_session_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_session_id(input);
        self
    }
    /// <p>The unique identifier of the Q App session the file is associated with, if applicable.</p>
    pub fn get_session_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_session_id()
    }
}
