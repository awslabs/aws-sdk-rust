// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct CreateApiKeyCredentialProviderOutput {
    /// <p>The Amazon Resource Name (ARN) of the secret containing the API key.</p>
    pub api_key_secret_arn: ::std::option::Option<crate::types::Secret>,
    /// <p>The name of the created API key credential provider.</p>
    pub name: ::std::string::String,
    /// <p>The Amazon Resource Name (ARN) of the created API key credential provider.</p>
    pub credential_provider_arn: ::std::string::String,
    _request_id: Option<String>,
}
impl CreateApiKeyCredentialProviderOutput {
    /// <p>The Amazon Resource Name (ARN) of the secret containing the API key.</p>
    pub fn api_key_secret_arn(&self) -> ::std::option::Option<&crate::types::Secret> {
        self.api_key_secret_arn.as_ref()
    }
    /// <p>The name of the created API key credential provider.</p>
    pub fn name(&self) -> &str {
        use std::ops::Deref;
        self.name.deref()
    }
    /// <p>The Amazon Resource Name (ARN) of the created API key credential provider.</p>
    pub fn credential_provider_arn(&self) -> &str {
        use std::ops::Deref;
        self.credential_provider_arn.deref()
    }
}
impl ::aws_types::request_id::RequestId for CreateApiKeyCredentialProviderOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl CreateApiKeyCredentialProviderOutput {
    /// Creates a new builder-style object to manufacture [`CreateApiKeyCredentialProviderOutput`](crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderOutput).
    pub fn builder() -> crate::operation::create_api_key_credential_provider::builders::CreateApiKeyCredentialProviderOutputBuilder {
        crate::operation::create_api_key_credential_provider::builders::CreateApiKeyCredentialProviderOutputBuilder::default()
    }
}

/// A builder for [`CreateApiKeyCredentialProviderOutput`](crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderOutput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct CreateApiKeyCredentialProviderOutputBuilder {
    pub(crate) api_key_secret_arn: ::std::option::Option<crate::types::Secret>,
    pub(crate) name: ::std::option::Option<::std::string::String>,
    pub(crate) credential_provider_arn: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl CreateApiKeyCredentialProviderOutputBuilder {
    /// <p>The Amazon Resource Name (ARN) of the secret containing the API key.</p>
    /// This field is required.
    pub fn api_key_secret_arn(mut self, input: crate::types::Secret) -> Self {
        self.api_key_secret_arn = ::std::option::Option::Some(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the secret containing the API key.</p>
    pub fn set_api_key_secret_arn(mut self, input: ::std::option::Option<crate::types::Secret>) -> Self {
        self.api_key_secret_arn = input;
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the secret containing the API key.</p>
    pub fn get_api_key_secret_arn(&self) -> &::std::option::Option<crate::types::Secret> {
        &self.api_key_secret_arn
    }
    /// <p>The name of the created API key credential provider.</p>
    /// This field is required.
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.name = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The name of the created API key credential provider.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.name = input;
        self
    }
    /// <p>The name of the created API key credential provider.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.name
    }
    /// <p>The Amazon Resource Name (ARN) of the created API key credential provider.</p>
    /// This field is required.
    pub fn credential_provider_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.credential_provider_arn = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the created API key credential provider.</p>
    pub fn set_credential_provider_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.credential_provider_arn = input;
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the created API key credential provider.</p>
    pub fn get_credential_provider_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.credential_provider_arn
    }
    pub(crate) fn _request_id(mut self, request_id: impl Into<String>) -> Self {
        self._request_id = Some(request_id.into());
        self
    }

    pub(crate) fn _set_request_id(&mut self, request_id: Option<String>) -> &mut Self {
        self._request_id = request_id;
        self
    }
    /// Consumes the builder and constructs a [`CreateApiKeyCredentialProviderOutput`](crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderOutput).
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](crate::operation::create_api_key_credential_provider::builders::CreateApiKeyCredentialProviderOutputBuilder::name)
    /// - [`credential_provider_arn`](crate::operation::create_api_key_credential_provider::builders::CreateApiKeyCredentialProviderOutputBuilder::credential_provider_arn)
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderOutput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(
            crate::operation::create_api_key_credential_provider::CreateApiKeyCredentialProviderOutput {
                api_key_secret_arn: self.api_key_secret_arn,
                name: self.name.ok_or_else(|| {
                    ::aws_smithy_types::error::operation::BuildError::missing_field(
                        "name",
                        "name was not specified but it is required when building CreateApiKeyCredentialProviderOutput",
                    )
                })?,
                credential_provider_arn: self.credential_provider_arn.ok_or_else(|| {
                    ::aws_smithy_types::error::operation::BuildError::missing_field(
                        "credential_provider_arn",
                        "credential_provider_arn was not specified but it is required when building CreateApiKeyCredentialProviderOutput",
                    )
                })?,
                _request_id: self._request_id,
            },
        )
    }
}
