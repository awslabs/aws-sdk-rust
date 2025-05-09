// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::reset_origin_endpoint_state::_reset_origin_endpoint_state_output::ResetOriginEndpointStateOutputBuilder;

pub use crate::operation::reset_origin_endpoint_state::_reset_origin_endpoint_state_input::ResetOriginEndpointStateInputBuilder;

impl crate::operation::reset_origin_endpoint_state::builders::ResetOriginEndpointStateInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::reset_origin_endpoint_state::ResetOriginEndpointStateOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::reset_origin_endpoint_state::ResetOriginEndpointStateError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.reset_origin_endpoint_state();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `ResetOriginEndpointState`.
///
/// <p>Resetting the origin endpoint can help to resolve unexpected behavior and other content packaging issues. It also helps to preserve special events when you don't want the previous content to be available for viewing. A reset clears out all previous content from the origin endpoint.</p>
/// <p>MediaPackage might return old content from this endpoint in the first 30 seconds after the endpoint reset. For best results, when possible, wait 30 seconds from endpoint reset to send playback requests to this endpoint.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct ResetOriginEndpointStateFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::reset_origin_endpoint_state::builders::ResetOriginEndpointStateInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::reset_origin_endpoint_state::ResetOriginEndpointStateOutput,
        crate::operation::reset_origin_endpoint_state::ResetOriginEndpointStateError,
    > for ResetOriginEndpointStateFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::reset_origin_endpoint_state::ResetOriginEndpointStateOutput,
            crate::operation::reset_origin_endpoint_state::ResetOriginEndpointStateError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl ResetOriginEndpointStateFluentBuilder {
    /// Creates a new `ResetOriginEndpointStateFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the ResetOriginEndpointState as a reference.
    pub fn as_input(&self) -> &crate::operation::reset_origin_endpoint_state::builders::ResetOriginEndpointStateInputBuilder {
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
        crate::operation::reset_origin_endpoint_state::ResetOriginEndpointStateOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::reset_origin_endpoint_state::ResetOriginEndpointStateError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::reset_origin_endpoint_state::ResetOriginEndpointState::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::reset_origin_endpoint_state::ResetOriginEndpointState::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::reset_origin_endpoint_state::ResetOriginEndpointStateOutput,
        crate::operation::reset_origin_endpoint_state::ResetOriginEndpointStateError,
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
    /// <p>The name of the channel group that contains the channel with the origin endpoint that you are resetting.</p>
    pub fn channel_group_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.channel_group_name(input.into());
        self
    }
    /// <p>The name of the channel group that contains the channel with the origin endpoint that you are resetting.</p>
    pub fn set_channel_group_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_channel_group_name(input);
        self
    }
    /// <p>The name of the channel group that contains the channel with the origin endpoint that you are resetting.</p>
    pub fn get_channel_group_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_channel_group_name()
    }
    /// <p>The name of the channel with the origin endpoint that you are resetting.</p>
    pub fn channel_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.channel_name(input.into());
        self
    }
    /// <p>The name of the channel with the origin endpoint that you are resetting.</p>
    pub fn set_channel_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_channel_name(input);
        self
    }
    /// <p>The name of the channel with the origin endpoint that you are resetting.</p>
    pub fn get_channel_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_channel_name()
    }
    /// <p>The name of the origin endpoint that you are resetting.</p>
    pub fn origin_endpoint_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.origin_endpoint_name(input.into());
        self
    }
    /// <p>The name of the origin endpoint that you are resetting.</p>
    pub fn set_origin_endpoint_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_origin_endpoint_name(input);
        self
    }
    /// <p>The name of the origin endpoint that you are resetting.</p>
    pub fn get_origin_endpoint_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_origin_endpoint_name()
    }
}
