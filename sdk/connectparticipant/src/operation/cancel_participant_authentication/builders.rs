// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::cancel_participant_authentication::_cancel_participant_authentication_output::CancelParticipantAuthenticationOutputBuilder;

pub use crate::operation::cancel_participant_authentication::_cancel_participant_authentication_input::CancelParticipantAuthenticationInputBuilder;

impl crate::operation::cancel_participant_authentication::builders::CancelParticipantAuthenticationInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::cancel_participant_authentication::CancelParticipantAuthenticationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::cancel_participant_authentication::CancelParticipantAuthenticationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.cancel_participant_authentication();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CancelParticipantAuthentication`.
///
/// <p>Cancels the authentication session. The opted out branch of the Authenticate Customer flow block will be taken.</p><note>
/// <p>The current supported channel is chat. This API is not supported for Apple Messages for Business, WhatsApp, or SMS chats.</p>
/// </note>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CancelParticipantAuthenticationFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::cancel_participant_authentication::builders::CancelParticipantAuthenticationInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::cancel_participant_authentication::CancelParticipantAuthenticationOutput,
        crate::operation::cancel_participant_authentication::CancelParticipantAuthenticationError,
    > for CancelParticipantAuthenticationFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::cancel_participant_authentication::CancelParticipantAuthenticationOutput,
            crate::operation::cancel_participant_authentication::CancelParticipantAuthenticationError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CancelParticipantAuthenticationFluentBuilder {
    /// Creates a new `CancelParticipantAuthenticationFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CancelParticipantAuthentication as a reference.
    pub fn as_input(&self) -> &crate::operation::cancel_participant_authentication::builders::CancelParticipantAuthenticationInputBuilder {
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
        crate::operation::cancel_participant_authentication::CancelParticipantAuthenticationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::cancel_participant_authentication::CancelParticipantAuthenticationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::cancel_participant_authentication::CancelParticipantAuthentication::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::cancel_participant_authentication::CancelParticipantAuthentication::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::cancel_participant_authentication::CancelParticipantAuthenticationOutput,
        crate::operation::cancel_participant_authentication::CancelParticipantAuthenticationError,
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
    /// <p>The <code>sessionId</code> provided in the <code>authenticationInitiated</code> event.</p>
    pub fn session_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.session_id(input.into());
        self
    }
    /// <p>The <code>sessionId</code> provided in the <code>authenticationInitiated</code> event.</p>
    pub fn set_session_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_session_id(input);
        self
    }
    /// <p>The <code>sessionId</code> provided in the <code>authenticationInitiated</code> event.</p>
    pub fn get_session_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_session_id()
    }
    /// <p>The authentication token associated with the participant's connection.</p>
    pub fn connection_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.connection_token(input.into());
        self
    }
    /// <p>The authentication token associated with the participant's connection.</p>
    pub fn set_connection_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_connection_token(input);
        self
    }
    /// <p>The authentication token associated with the participant's connection.</p>
    pub fn get_connection_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_connection_token()
    }
}
