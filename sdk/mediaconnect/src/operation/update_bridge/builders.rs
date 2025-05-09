// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::update_bridge::_update_bridge_output::UpdateBridgeOutputBuilder;

pub use crate::operation::update_bridge::_update_bridge_input::UpdateBridgeInputBuilder;

impl crate::operation::update_bridge::builders::UpdateBridgeInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::update_bridge::UpdateBridgeOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_bridge::UpdateBridgeError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.update_bridge();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UpdateBridge`.
///
/// <p>Updates the bridge.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UpdateBridgeFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::update_bridge::builders::UpdateBridgeInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::update_bridge::UpdateBridgeOutput,
        crate::operation::update_bridge::UpdateBridgeError,
    > for UpdateBridgeFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::update_bridge::UpdateBridgeOutput,
            crate::operation::update_bridge::UpdateBridgeError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UpdateBridgeFluentBuilder {
    /// Creates a new `UpdateBridgeFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UpdateBridge as a reference.
    pub fn as_input(&self) -> &crate::operation::update_bridge::builders::UpdateBridgeInputBuilder {
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
        crate::operation::update_bridge::UpdateBridgeOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_bridge::UpdateBridgeError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::update_bridge::UpdateBridge::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::update_bridge::UpdateBridge::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::update_bridge::UpdateBridgeOutput,
        crate::operation::update_bridge::UpdateBridgeError,
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
    /// <p>TheAmazon Resource Name (ARN) of the bridge that you want to update.</p>
    pub fn bridge_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.bridge_arn(input.into());
        self
    }
    /// <p>TheAmazon Resource Name (ARN) of the bridge that you want to update.</p>
    pub fn set_bridge_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_bridge_arn(input);
        self
    }
    /// <p>TheAmazon Resource Name (ARN) of the bridge that you want to update.</p>
    pub fn get_bridge_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_bridge_arn()
    }
    /// <p>A cloud-to-ground bridge. The content comes from an existing MediaConnect flow and is delivered to your premises.</p>
    pub fn egress_gateway_bridge(mut self, input: crate::types::UpdateEgressGatewayBridgeRequest) -> Self {
        self.inner = self.inner.egress_gateway_bridge(input);
        self
    }
    /// <p>A cloud-to-ground bridge. The content comes from an existing MediaConnect flow and is delivered to your premises.</p>
    pub fn set_egress_gateway_bridge(mut self, input: ::std::option::Option<crate::types::UpdateEgressGatewayBridgeRequest>) -> Self {
        self.inner = self.inner.set_egress_gateway_bridge(input);
        self
    }
    /// <p>A cloud-to-ground bridge. The content comes from an existing MediaConnect flow and is delivered to your premises.</p>
    pub fn get_egress_gateway_bridge(&self) -> &::std::option::Option<crate::types::UpdateEgressGatewayBridgeRequest> {
        self.inner.get_egress_gateway_bridge()
    }
    /// <p>A ground-to-cloud bridge. The content originates at your premises and is delivered to the cloud.</p>
    pub fn ingress_gateway_bridge(mut self, input: crate::types::UpdateIngressGatewayBridgeRequest) -> Self {
        self.inner = self.inner.ingress_gateway_bridge(input);
        self
    }
    /// <p>A ground-to-cloud bridge. The content originates at your premises and is delivered to the cloud.</p>
    pub fn set_ingress_gateway_bridge(mut self, input: ::std::option::Option<crate::types::UpdateIngressGatewayBridgeRequest>) -> Self {
        self.inner = self.inner.set_ingress_gateway_bridge(input);
        self
    }
    /// <p>A ground-to-cloud bridge. The content originates at your premises and is delivered to the cloud.</p>
    pub fn get_ingress_gateway_bridge(&self) -> &::std::option::Option<crate::types::UpdateIngressGatewayBridgeRequest> {
        self.inner.get_ingress_gateway_bridge()
    }
    /// <p>The settings for source failover.</p>
    pub fn source_failover_config(mut self, input: crate::types::UpdateFailoverConfig) -> Self {
        self.inner = self.inner.source_failover_config(input);
        self
    }
    /// <p>The settings for source failover.</p>
    pub fn set_source_failover_config(mut self, input: ::std::option::Option<crate::types::UpdateFailoverConfig>) -> Self {
        self.inner = self.inner.set_source_failover_config(input);
        self
    }
    /// <p>The settings for source failover.</p>
    pub fn get_source_failover_config(&self) -> &::std::option::Option<crate::types::UpdateFailoverConfig> {
        self.inner.get_source_failover_config()
    }
}
