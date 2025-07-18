// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_space::_create_space_output::CreateSpaceOutputBuilder;

pub use crate::operation::create_space::_create_space_input::CreateSpaceInputBuilder;

impl crate::operation::create_space::builders::CreateSpaceInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_space::CreateSpaceOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_space::CreateSpaceError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_space();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateSpace`.
///
/// <p>Creates an AWS re:Post Private private re:Post.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateSpaceFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_space::builders::CreateSpaceInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_space::CreateSpaceOutput,
        crate::operation::create_space::CreateSpaceError,
    > for CreateSpaceFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_space::CreateSpaceOutput,
            crate::operation::create_space::CreateSpaceError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateSpaceFluentBuilder {
    /// Creates a new `CreateSpaceFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateSpace as a reference.
    pub fn as_input(&self) -> &crate::operation::create_space::builders::CreateSpaceInputBuilder {
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
        crate::operation::create_space::CreateSpaceOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_space::CreateSpaceError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_space::CreateSpace::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_space::CreateSpace::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_space::CreateSpaceOutput,
        crate::operation::create_space::CreateSpaceError,
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
    /// <p>The name for the private re:Post. This must be unique in your account.</p>
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.name(input.into());
        self
    }
    /// <p>The name for the private re:Post. This must be unique in your account.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_name(input);
        self
    }
    /// <p>The name for the private re:Post. This must be unique in your account.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_name()
    }
    /// <p>The subdomain that you use to access your AWS re:Post Private private re:Post. All custom subdomains must be approved by AWS before use. In addition to your custom subdomain, all private re:Posts are issued an AWS generated subdomain for immediate use.</p>
    pub fn subdomain(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.subdomain(input.into());
        self
    }
    /// <p>The subdomain that you use to access your AWS re:Post Private private re:Post. All custom subdomains must be approved by AWS before use. In addition to your custom subdomain, all private re:Posts are issued an AWS generated subdomain for immediate use.</p>
    pub fn set_subdomain(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_subdomain(input);
        self
    }
    /// <p>The subdomain that you use to access your AWS re:Post Private private re:Post. All custom subdomains must be approved by AWS before use. In addition to your custom subdomain, all private re:Posts are issued an AWS generated subdomain for immediate use.</p>
    pub fn get_subdomain(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_subdomain()
    }
    /// <p>The pricing tier for the private re:Post.</p>
    pub fn tier(mut self, input: crate::types::TierLevel) -> Self {
        self.inner = self.inner.tier(input);
        self
    }
    /// <p>The pricing tier for the private re:Post.</p>
    pub fn set_tier(mut self, input: ::std::option::Option<crate::types::TierLevel>) -> Self {
        self.inner = self.inner.set_tier(input);
        self
    }
    /// <p>The pricing tier for the private re:Post.</p>
    pub fn get_tier(&self) -> &::std::option::Option<crate::types::TierLevel> {
        self.inner.get_tier()
    }
    /// <p>A description for the private re:Post. This is used only to help you identify this private re:Post.</p>
    pub fn description(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.description(input.into());
        self
    }
    /// <p>A description for the private re:Post. This is used only to help you identify this private re:Post.</p>
    pub fn set_description(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_description(input);
        self
    }
    /// <p>A description for the private re:Post. This is used only to help you identify this private re:Post.</p>
    pub fn get_description(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_description()
    }
    /// <p>The AWS KMS key ARN that’s used for the AWS KMS encryption. If you don't provide a key, your data is encrypted by default with a key that AWS owns and manages for you.</p>
    pub fn user_kms_key(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.user_kms_key(input.into());
        self
    }
    /// <p>The AWS KMS key ARN that’s used for the AWS KMS encryption. If you don't provide a key, your data is encrypted by default with a key that AWS owns and manages for you.</p>
    pub fn set_user_kms_key(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_user_kms_key(input);
        self
    }
    /// <p>The AWS KMS key ARN that’s used for the AWS KMS encryption. If you don't provide a key, your data is encrypted by default with a key that AWS owns and manages for you.</p>
    pub fn get_user_kms_key(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_user_kms_key()
    }
    ///
    /// Adds a key-value pair to `tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>The list of tags associated with the private re:Post.</p>
    pub fn tags(mut self, k: impl ::std::convert::Into<::std::string::String>, v: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.tags(k.into(), v.into());
        self
    }
    /// <p>The list of tags associated with the private re:Post.</p>
    pub fn set_tags(mut self, input: ::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>The list of tags associated with the private re:Post.</p>
    pub fn get_tags(&self) -> &::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>> {
        self.inner.get_tags()
    }
    /// <p>The IAM role that grants permissions to the private re:Post to convert unanswered questions into AWS support tickets.</p>
    pub fn role_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.role_arn(input.into());
        self
    }
    /// <p>The IAM role that grants permissions to the private re:Post to convert unanswered questions into AWS support tickets.</p>
    pub fn set_role_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_role_arn(input);
        self
    }
    /// <p>The IAM role that grants permissions to the private re:Post to convert unanswered questions into AWS support tickets.</p>
    pub fn get_role_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_role_arn()
    }
    /// <p></p>
    pub fn supported_email_domains(mut self, input: crate::types::SupportedEmailDomainsParameters) -> Self {
        self.inner = self.inner.supported_email_domains(input);
        self
    }
    /// <p></p>
    pub fn set_supported_email_domains(mut self, input: ::std::option::Option<crate::types::SupportedEmailDomainsParameters>) -> Self {
        self.inner = self.inner.set_supported_email_domains(input);
        self
    }
    /// <p></p>
    pub fn get_supported_email_domains(&self) -> &::std::option::Option<crate::types::SupportedEmailDomainsParameters> {
        self.inner.get_supported_email_domains()
    }
}
