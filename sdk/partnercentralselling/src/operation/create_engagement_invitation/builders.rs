// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_engagement_invitation::_create_engagement_invitation_output::CreateEngagementInvitationOutputBuilder;

pub use crate::operation::create_engagement_invitation::_create_engagement_invitation_input::CreateEngagementInvitationInputBuilder;

impl crate::operation::create_engagement_invitation::builders::CreateEngagementInvitationInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_engagement_invitation::CreateEngagementInvitationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_engagement_invitation::CreateEngagementInvitationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_engagement_invitation();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateEngagementInvitation`.
///
/// <p>This action creates an invitation from a sender to a single receiver to join an engagement.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateEngagementInvitationFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_engagement_invitation::builders::CreateEngagementInvitationInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_engagement_invitation::CreateEngagementInvitationOutput,
        crate::operation::create_engagement_invitation::CreateEngagementInvitationError,
    > for CreateEngagementInvitationFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_engagement_invitation::CreateEngagementInvitationOutput,
            crate::operation::create_engagement_invitation::CreateEngagementInvitationError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateEngagementInvitationFluentBuilder {
    /// Creates a new `CreateEngagementInvitationFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateEngagementInvitation as a reference.
    pub fn as_input(&self) -> &crate::operation::create_engagement_invitation::builders::CreateEngagementInvitationInputBuilder {
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
        crate::operation::create_engagement_invitation::CreateEngagementInvitationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_engagement_invitation::CreateEngagementInvitationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_engagement_invitation::CreateEngagementInvitation::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_engagement_invitation::CreateEngagementInvitation::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_engagement_invitation::CreateEngagementInvitationOutput,
        crate::operation::create_engagement_invitation::CreateEngagementInvitationError,
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
    /// <p>Specifies the catalog related to the engagement. Accepted values are <code>AWS</code> and <code>Sandbox</code>, which determine the environment in which the engagement is managed.</p>
    pub fn catalog(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.catalog(input.into());
        self
    }
    /// <p>Specifies the catalog related to the engagement. Accepted values are <code>AWS</code> and <code>Sandbox</code>, which determine the environment in which the engagement is managed.</p>
    pub fn set_catalog(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_catalog(input);
        self
    }
    /// <p>Specifies the catalog related to the engagement. Accepted values are <code>AWS</code> and <code>Sandbox</code>, which determine the environment in which the engagement is managed.</p>
    pub fn get_catalog(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_catalog()
    }
    /// <p>Specifies a unique, client-generated UUID to ensure that the request is handled exactly once. This token helps prevent duplicate invitation creations.</p>
    pub fn client_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_token(input.into());
        self
    }
    /// <p>Specifies a unique, client-generated UUID to ensure that the request is handled exactly once. This token helps prevent duplicate invitation creations.</p>
    pub fn set_client_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_token(input);
        self
    }
    /// <p>Specifies a unique, client-generated UUID to ensure that the request is handled exactly once. This token helps prevent duplicate invitation creations.</p>
    pub fn get_client_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_token()
    }
    /// <p>The unique identifier of the <code>Engagement</code> associated with the invitation. This parameter ensures the invitation is created within the correct <code>Engagement</code> context.</p>
    pub fn engagement_identifier(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.engagement_identifier(input.into());
        self
    }
    /// <p>The unique identifier of the <code>Engagement</code> associated with the invitation. This parameter ensures the invitation is created within the correct <code>Engagement</code> context.</p>
    pub fn set_engagement_identifier(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_engagement_identifier(input);
        self
    }
    /// <p>The unique identifier of the <code>Engagement</code> associated with the invitation. This parameter ensures the invitation is created within the correct <code>Engagement</code> context.</p>
    pub fn get_engagement_identifier(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_engagement_identifier()
    }
    /// <p>The <code>Invitation</code> object all information necessary to initiate an engagement invitation to a partner. It contains a personalized message from the sender, the invitation's receiver, and a payload. The <code>Payload</code> can be the <code>OpportunityInvitation</code>, which includes detailed structures for sender contacts, partner responsibilities, customer information, and project details.</p>
    pub fn invitation(mut self, input: crate::types::Invitation) -> Self {
        self.inner = self.inner.invitation(input);
        self
    }
    /// <p>The <code>Invitation</code> object all information necessary to initiate an engagement invitation to a partner. It contains a personalized message from the sender, the invitation's receiver, and a payload. The <code>Payload</code> can be the <code>OpportunityInvitation</code>, which includes detailed structures for sender contacts, partner responsibilities, customer information, and project details.</p>
    pub fn set_invitation(mut self, input: ::std::option::Option<crate::types::Invitation>) -> Self {
        self.inner = self.inner.set_invitation(input);
        self
    }
    /// <p>The <code>Invitation</code> object all information necessary to initiate an engagement invitation to a partner. It contains a personalized message from the sender, the invitation's receiver, and a payload. The <code>Payload</code> can be the <code>OpportunityInvitation</code>, which includes detailed structures for sender contacts, partner responsibilities, customer information, and project details.</p>
    pub fn get_invitation(&self) -> &::std::option::Option<crate::types::Invitation> {
        self.inner.get_invitation()
    }
}
