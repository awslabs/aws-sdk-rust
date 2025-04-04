// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::add_flow_media_streams::_add_flow_media_streams_output::AddFlowMediaStreamsOutputBuilder;

pub use crate::operation::add_flow_media_streams::_add_flow_media_streams_input::AddFlowMediaStreamsInputBuilder;

impl crate::operation::add_flow_media_streams::builders::AddFlowMediaStreamsInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::add_flow_media_streams::AddFlowMediaStreamsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::add_flow_media_streams::AddFlowMediaStreamsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.add_flow_media_streams();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `AddFlowMediaStreams`.
///
/// <p>Adds media streams to an existing flow. After you add a media stream to a flow, you can associate it with a source and/or an output that uses the ST 2110 JPEG XS or CDI protocol.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct AddFlowMediaStreamsFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::add_flow_media_streams::builders::AddFlowMediaStreamsInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::add_flow_media_streams::AddFlowMediaStreamsOutput,
        crate::operation::add_flow_media_streams::AddFlowMediaStreamsError,
    > for AddFlowMediaStreamsFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::add_flow_media_streams::AddFlowMediaStreamsOutput,
            crate::operation::add_flow_media_streams::AddFlowMediaStreamsError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl AddFlowMediaStreamsFluentBuilder {
    /// Creates a new `AddFlowMediaStreamsFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the AddFlowMediaStreams as a reference.
    pub fn as_input(&self) -> &crate::operation::add_flow_media_streams::builders::AddFlowMediaStreamsInputBuilder {
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
        crate::operation::add_flow_media_streams::AddFlowMediaStreamsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::add_flow_media_streams::AddFlowMediaStreamsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::add_flow_media_streams::AddFlowMediaStreams::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::add_flow_media_streams::AddFlowMediaStreams::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::add_flow_media_streams::AddFlowMediaStreamsOutput,
        crate::operation::add_flow_media_streams::AddFlowMediaStreamsError,
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
    /// <p>The Amazon Resource Name (ARN) of the flow.</p>
    pub fn flow_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.flow_arn(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the flow.</p>
    pub fn set_flow_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_flow_arn(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the flow.</p>
    pub fn get_flow_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_flow_arn()
    }
    ///
    /// Appends an item to `MediaStreams`.
    ///
    /// To override the contents of this collection use [`set_media_streams`](Self::set_media_streams).
    ///
    /// <p>The media streams that you want to add to the flow.</p>
    pub fn media_streams(mut self, input: crate::types::AddMediaStreamRequest) -> Self {
        self.inner = self.inner.media_streams(input);
        self
    }
    /// <p>The media streams that you want to add to the flow.</p>
    pub fn set_media_streams(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::AddMediaStreamRequest>>) -> Self {
        self.inner = self.inner.set_media_streams(input);
        self
    }
    /// <p>The media streams that you want to add to the flow.</p>
    pub fn get_media_streams(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::AddMediaStreamRequest>> {
        self.inner.get_media_streams()
    }
}
