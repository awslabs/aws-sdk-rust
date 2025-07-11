// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_notification_configuration::_create_notification_configuration_output::CreateNotificationConfigurationOutputBuilder;

pub use crate::operation::create_notification_configuration::_create_notification_configuration_input::CreateNotificationConfigurationInputBuilder;

impl crate::operation::create_notification_configuration::builders::CreateNotificationConfigurationInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_notification_configuration::CreateNotificationConfigurationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_notification_configuration::CreateNotificationConfigurationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_notification_configuration();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateNotificationConfiguration`.
///
/// <p>Creates a notification configuration. A configuration is a connection between an event type and a destination that you have already created.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateNotificationConfigurationFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_notification_configuration::builders::CreateNotificationConfigurationInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_notification_configuration::CreateNotificationConfigurationOutput,
        crate::operation::create_notification_configuration::CreateNotificationConfigurationError,
    > for CreateNotificationConfigurationFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_notification_configuration::CreateNotificationConfigurationOutput,
            crate::operation::create_notification_configuration::CreateNotificationConfigurationError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateNotificationConfigurationFluentBuilder {
    /// Creates a new `CreateNotificationConfigurationFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateNotificationConfiguration as a reference.
    pub fn as_input(&self) -> &crate::operation::create_notification_configuration::builders::CreateNotificationConfigurationInputBuilder {
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
        crate::operation::create_notification_configuration::CreateNotificationConfigurationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_notification_configuration::CreateNotificationConfigurationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_notification_configuration::CreateNotificationConfiguration::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_notification_configuration::CreateNotificationConfiguration::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_notification_configuration::CreateNotificationConfigurationOutput,
        crate::operation::create_notification_configuration::CreateNotificationConfigurationError,
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
    /// <p>The type of event triggering a device notification to the customer-managed destination.</p>
    pub fn event_type(mut self, input: crate::types::EventType) -> Self {
        self.inner = self.inner.event_type(input);
        self
    }
    /// <p>The type of event triggering a device notification to the customer-managed destination.</p>
    pub fn set_event_type(mut self, input: ::std::option::Option<crate::types::EventType>) -> Self {
        self.inner = self.inner.set_event_type(input);
        self
    }
    /// <p>The type of event triggering a device notification to the customer-managed destination.</p>
    pub fn get_event_type(&self) -> &::std::option::Option<crate::types::EventType> {
        self.inner.get_event_type()
    }
    /// <p>The name of the destination for the notification configuration.</p>
    pub fn destination_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.destination_name(input.into());
        self
    }
    /// <p>The name of the destination for the notification configuration.</p>
    pub fn set_destination_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_destination_name(input);
        self
    }
    /// <p>The name of the destination for the notification configuration.</p>
    pub fn get_destination_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_destination_name()
    }
    /// <p>An idempotency token. If you retry a request that completed successfully initially using the same client token and parameters, then the retry attempt will succeed without performing any further actions.</p>
    pub fn client_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_token(input.into());
        self
    }
    /// <p>An idempotency token. If you retry a request that completed successfully initially using the same client token and parameters, then the retry attempt will succeed without performing any further actions.</p>
    pub fn set_client_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_token(input);
        self
    }
    /// <p>An idempotency token. If you retry a request that completed successfully initially using the same client token and parameters, then the retry attempt will succeed without performing any further actions.</p>
    pub fn get_client_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_token()
    }
    ///
    /// Adds a key-value pair to `Tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>A set of key/value pairs that are used to manage the notification configuration.</p>
    #[deprecated(note = "Tags has been deprecated from this api", since = "06-25-2025")]
    pub fn tags(mut self, k: impl ::std::convert::Into<::std::string::String>, v: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.tags(k.into(), v.into());
        self
    }
    /// <p>A set of key/value pairs that are used to manage the notification configuration.</p>
    #[deprecated(note = "Tags has been deprecated from this api", since = "06-25-2025")]
    pub fn set_tags(mut self, input: ::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>A set of key/value pairs that are used to manage the notification configuration.</p>
    #[deprecated(note = "Tags has been deprecated from this api", since = "06-25-2025")]
    pub fn get_tags(&self) -> &::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>> {
        self.inner.get_tags()
    }
}
