// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_api_key_credential_provider::_create_api_key_credential_provider_output::CreateApiKeyCredentialProviderOutputBuilder;

pub use crate::operation::create_api_key_credential_provider::_create_api_key_credential_provider_input::CreateApiKeyCredentialProviderInputBuilder;

impl crate::operation::create_api_key_credential_provider::builders::CreateApiKeyCredentialProviderInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_api_key_credential_provider();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateApiKeyCredentialProvider`.
///
/// <p>Creates a new API key credential provider.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateApiKeyCredentialProviderFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_api_key_credential_provider::builders::CreateApiKeyCredentialProviderInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderOutput,
        crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderError,
    > for CreateApiKeyCredentialProviderFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderOutput,
            crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateApiKeyCredentialProviderFluentBuilder {
    /// Creates a new `CreateApiKeyCredentialProviderFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateApiKeyCredentialProvider as a reference.
    pub fn as_input(&self) -> &crate::operation::create_api_key_credential_provider::builders::CreateApiKeyCredentialProviderInputBuilder {
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
        crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProvider::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProvider::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderOutput,
        crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderError,
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
    /// <p>The name of the API key credential provider. The name must be unique within your account.</p>
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.name(input.into());
        self
    }
    /// <p>The name of the API key credential provider. The name must be unique within your account.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_name(input);
        self
    }
    /// <p>The name of the API key credential provider. The name must be unique within your account.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_name()
    }
    /// <p>The API key to use for authentication. This value is encrypted and stored securely.</p>
    pub fn api_key(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.api_key(input.into());
        self
    }
    /// <p>The API key to use for authentication. This value is encrypted and stored securely.</p>
    pub fn set_api_key(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_api_key(input);
        self
    }
    /// <p>The API key to use for authentication. This value is encrypted and stored securely.</p>
    pub fn get_api_key(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_api_key()
    }
}
