// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::update_dataset::_update_dataset_output::UpdateDatasetOutputBuilder;

pub use crate::operation::update_dataset::_update_dataset_input::UpdateDatasetInputBuilder;

impl crate::operation::update_dataset::builders::UpdateDatasetInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::update_dataset::UpdateDatasetOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_dataset::UpdateDatasetError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.update_dataset();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UpdateDataset`.
///
/// <p>Updates a dataset.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UpdateDatasetFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::update_dataset::builders::UpdateDatasetInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::update_dataset::UpdateDatasetOutput,
        crate::operation::update_dataset::UpdateDatasetError,
    > for UpdateDatasetFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::update_dataset::UpdateDatasetOutput,
            crate::operation::update_dataset::UpdateDatasetError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UpdateDatasetFluentBuilder {
    /// Creates a new `UpdateDatasetFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UpdateDataset as a reference.
    pub fn as_input(&self) -> &crate::operation::update_dataset::builders::UpdateDatasetInputBuilder {
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
        crate::operation::update_dataset::UpdateDatasetOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_dataset::UpdateDatasetError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::update_dataset::UpdateDataset::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::update_dataset::UpdateDataset::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::update_dataset::UpdateDatasetOutput,
        crate::operation::update_dataset::UpdateDatasetError,
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
    /// <p>The ID of the dataset.</p>
    pub fn dataset_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.dataset_id(input.into());
        self
    }
    /// <p>The ID of the dataset.</p>
    pub fn set_dataset_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_dataset_id(input);
        self
    }
    /// <p>The ID of the dataset.</p>
    pub fn get_dataset_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_dataset_id()
    }
    /// <p>The name of the dataset.</p>
    pub fn dataset_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.dataset_name(input.into());
        self
    }
    /// <p>The name of the dataset.</p>
    pub fn set_dataset_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_dataset_name(input);
        self
    }
    /// <p>The name of the dataset.</p>
    pub fn get_dataset_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_dataset_name()
    }
    /// <p>A description about the dataset, and its functionality.</p>
    pub fn dataset_description(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.dataset_description(input.into());
        self
    }
    /// <p>A description about the dataset, and its functionality.</p>
    pub fn set_dataset_description(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_dataset_description(input);
        self
    }
    /// <p>A description about the dataset, and its functionality.</p>
    pub fn get_dataset_description(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_dataset_description()
    }
    /// <p>The data source for the dataset.</p>
    pub fn dataset_source(mut self, input: crate::types::DatasetSource) -> Self {
        self.inner = self.inner.dataset_source(input);
        self
    }
    /// <p>The data source for the dataset.</p>
    pub fn set_dataset_source(mut self, input: ::std::option::Option<crate::types::DatasetSource>) -> Self {
        self.inner = self.inner.set_dataset_source(input);
        self
    }
    /// <p>The data source for the dataset.</p>
    pub fn get_dataset_source(&self) -> &::std::option::Option<crate::types::DatasetSource> {
        self.inner.get_dataset_source()
    }
    /// <p>A unique case-sensitive identifier that you can provide to ensure the idempotency of the request. Don't reuse this client token if a new idempotent request is required.</p>
    pub fn client_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_token(input.into());
        self
    }
    /// <p>A unique case-sensitive identifier that you can provide to ensure the idempotency of the request. Don't reuse this client token if a new idempotent request is required.</p>
    pub fn set_client_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_token(input);
        self
    }
    /// <p>A unique case-sensitive identifier that you can provide to ensure the idempotency of the request. Don't reuse this client token if a new idempotent request is required.</p>
    pub fn get_client_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_token()
    }
}
