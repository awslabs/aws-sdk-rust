// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_license_endpoint::_create_license_endpoint_output::CreateLicenseEndpointOutputBuilder;

pub use crate::operation::create_license_endpoint::_create_license_endpoint_input::CreateLicenseEndpointInputBuilder;

impl crate::operation::create_license_endpoint::builders::CreateLicenseEndpointInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_license_endpoint::CreateLicenseEndpointOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_license_endpoint::CreateLicenseEndpointError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_license_endpoint();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateLicenseEndpoint`.
///
/// <p>Creates a license endpoint to integrate your various licensed software used for rendering on Deadline Cloud.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateLicenseEndpointFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_license_endpoint::builders::CreateLicenseEndpointInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_license_endpoint::CreateLicenseEndpointOutput,
        crate::operation::create_license_endpoint::CreateLicenseEndpointError,
    > for CreateLicenseEndpointFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_license_endpoint::CreateLicenseEndpointOutput,
            crate::operation::create_license_endpoint::CreateLicenseEndpointError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateLicenseEndpointFluentBuilder {
    /// Creates a new `CreateLicenseEndpointFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateLicenseEndpoint as a reference.
    pub fn as_input(&self) -> &crate::operation::create_license_endpoint::builders::CreateLicenseEndpointInputBuilder {
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
        crate::operation::create_license_endpoint::CreateLicenseEndpointOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_license_endpoint::CreateLicenseEndpointError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_license_endpoint::CreateLicenseEndpoint::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_license_endpoint::CreateLicenseEndpoint::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_license_endpoint::CreateLicenseEndpointOutput,
        crate::operation::create_license_endpoint::CreateLicenseEndpointError,
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
    /// <p>The unique token which the server uses to recognize retries of the same request.</p>
    pub fn client_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_token(input.into());
        self
    }
    /// <p>The unique token which the server uses to recognize retries of the same request.</p>
    pub fn set_client_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_token(input);
        self
    }
    /// <p>The unique token which the server uses to recognize retries of the same request.</p>
    pub fn get_client_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_token()
    }
    /// <p>The VPC (virtual private cloud) ID to use with the license endpoint.</p>
    pub fn vpc_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.vpc_id(input.into());
        self
    }
    /// <p>The VPC (virtual private cloud) ID to use with the license endpoint.</p>
    pub fn set_vpc_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_vpc_id(input);
        self
    }
    /// <p>The VPC (virtual private cloud) ID to use with the license endpoint.</p>
    pub fn get_vpc_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_vpc_id()
    }
    ///
    /// Appends an item to `subnetIds`.
    ///
    /// To override the contents of this collection use [`set_subnet_ids`](Self::set_subnet_ids).
    ///
    /// <p>The subnet IDs.</p>
    pub fn subnet_ids(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.subnet_ids(input.into());
        self
    }
    /// <p>The subnet IDs.</p>
    pub fn set_subnet_ids(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_subnet_ids(input);
        self
    }
    /// <p>The subnet IDs.</p>
    pub fn get_subnet_ids(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_subnet_ids()
    }
    ///
    /// Appends an item to `securityGroupIds`.
    ///
    /// To override the contents of this collection use [`set_security_group_ids`](Self::set_security_group_ids).
    ///
    /// <p>The security group IDs.</p>
    pub fn security_group_ids(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.security_group_ids(input.into());
        self
    }
    /// <p>The security group IDs.</p>
    pub fn set_security_group_ids(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_security_group_ids(input);
        self
    }
    /// <p>The security group IDs.</p>
    pub fn get_security_group_ids(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_security_group_ids()
    }
    ///
    /// Adds a key-value pair to `tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>Each tag consists of a tag key and a tag value. Tag keys and values are both required, but tag values can be empty strings.</p>
    pub fn tags(mut self, k: impl ::std::convert::Into<::std::string::String>, v: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.tags(k.into(), v.into());
        self
    }
    /// <p>Each tag consists of a tag key and a tag value. Tag keys and values are both required, but tag values can be empty strings.</p>
    pub fn set_tags(mut self, input: ::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>Each tag consists of a tag key and a tag value. Tag keys and values are both required, but tag values can be empty strings.</p>
    pub fn get_tags(&self) -> &::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>> {
        self.inner.get_tags()
    }
}
