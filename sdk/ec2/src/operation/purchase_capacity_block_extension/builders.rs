// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::purchase_capacity_block_extension::_purchase_capacity_block_extension_output::PurchaseCapacityBlockExtensionOutputBuilder;

pub use crate::operation::purchase_capacity_block_extension::_purchase_capacity_block_extension_input::PurchaseCapacityBlockExtensionInputBuilder;

impl crate::operation::purchase_capacity_block_extension::builders::PurchaseCapacityBlockExtensionInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtensionOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtensionError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.purchase_capacity_block_extension();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `PurchaseCapacityBlockExtension`.
///
/// <p>Purchase the Capacity Block extension for use with your account. You must specify the ID of the Capacity Block extension offering you are purchasing.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct PurchaseCapacityBlockExtensionFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::purchase_capacity_block_extension::builders::PurchaseCapacityBlockExtensionInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtensionOutput,
        crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtensionError,
    > for PurchaseCapacityBlockExtensionFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtensionOutput,
            crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtensionError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl PurchaseCapacityBlockExtensionFluentBuilder {
    /// Creates a new `PurchaseCapacityBlockExtensionFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the PurchaseCapacityBlockExtension as a reference.
    pub fn as_input(&self) -> &crate::operation::purchase_capacity_block_extension::builders::PurchaseCapacityBlockExtensionInputBuilder {
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
        crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtensionOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtensionError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtension::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtension::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtensionOutput,
        crate::operation::purchase_capacity_block_extension::PurchaseCapacityBlockExtensionError,
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
    /// <p>The ID of the Capacity Block extension offering to purchase.</p>
    pub fn capacity_block_extension_offering_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.capacity_block_extension_offering_id(input.into());
        self
    }
    /// <p>The ID of the Capacity Block extension offering to purchase.</p>
    pub fn set_capacity_block_extension_offering_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_capacity_block_extension_offering_id(input);
        self
    }
    /// <p>The ID of the Capacity Block extension offering to purchase.</p>
    pub fn get_capacity_block_extension_offering_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_capacity_block_extension_offering_id()
    }
    /// <p>The ID of the Capacity reservation to be extended.</p>
    pub fn capacity_reservation_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.capacity_reservation_id(input.into());
        self
    }
    /// <p>The ID of the Capacity reservation to be extended.</p>
    pub fn set_capacity_reservation_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_capacity_reservation_id(input);
        self
    }
    /// <p>The ID of the Capacity reservation to be extended.</p>
    pub fn get_capacity_reservation_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_capacity_reservation_id()
    }
    /// <p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p>
    pub fn dry_run(mut self, input: bool) -> Self {
        self.inner = self.inner.dry_run(input);
        self
    }
    /// <p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p>
    pub fn set_dry_run(mut self, input: ::std::option::Option<bool>) -> Self {
        self.inner = self.inner.set_dry_run(input);
        self
    }
    /// <p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p>
    pub fn get_dry_run(&self) -> &::std::option::Option<bool> {
        self.inner.get_dry_run()
    }
}
