// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::update_compute_quota::_update_compute_quota_output::UpdateComputeQuotaOutputBuilder;

pub use crate::operation::update_compute_quota::_update_compute_quota_input::UpdateComputeQuotaInputBuilder;

impl crate::operation::update_compute_quota::builders::UpdateComputeQuotaInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::update_compute_quota::UpdateComputeQuotaOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_compute_quota::UpdateComputeQuotaError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.update_compute_quota();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UpdateComputeQuota`.
///
/// <p>Update the compute allocation definition.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UpdateComputeQuotaFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::update_compute_quota::builders::UpdateComputeQuotaInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::update_compute_quota::UpdateComputeQuotaOutput,
        crate::operation::update_compute_quota::UpdateComputeQuotaError,
    > for UpdateComputeQuotaFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::update_compute_quota::UpdateComputeQuotaOutput,
            crate::operation::update_compute_quota::UpdateComputeQuotaError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UpdateComputeQuotaFluentBuilder {
    /// Creates a new `UpdateComputeQuotaFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UpdateComputeQuota as a reference.
    pub fn as_input(&self) -> &crate::operation::update_compute_quota::builders::UpdateComputeQuotaInputBuilder {
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
        crate::operation::update_compute_quota::UpdateComputeQuotaOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_compute_quota::UpdateComputeQuotaError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::update_compute_quota::UpdateComputeQuota::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::update_compute_quota::UpdateComputeQuota::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::update_compute_quota::UpdateComputeQuotaOutput,
        crate::operation::update_compute_quota::UpdateComputeQuotaError,
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
    /// <p>ID of the compute allocation definition.</p>
    pub fn compute_quota_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.compute_quota_id(input.into());
        self
    }
    /// <p>ID of the compute allocation definition.</p>
    pub fn set_compute_quota_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_compute_quota_id(input);
        self
    }
    /// <p>ID of the compute allocation definition.</p>
    pub fn get_compute_quota_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_compute_quota_id()
    }
    /// <p>Target version.</p>
    pub fn target_version(mut self, input: i32) -> Self {
        self.inner = self.inner.target_version(input);
        self
    }
    /// <p>Target version.</p>
    pub fn set_target_version(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_target_version(input);
        self
    }
    /// <p>Target version.</p>
    pub fn get_target_version(&self) -> &::std::option::Option<i32> {
        self.inner.get_target_version()
    }
    /// <p>Configuration of the compute allocation definition. This includes the resource sharing option, and the setting to preempt low priority tasks.</p>
    pub fn compute_quota_config(mut self, input: crate::types::ComputeQuotaConfig) -> Self {
        self.inner = self.inner.compute_quota_config(input);
        self
    }
    /// <p>Configuration of the compute allocation definition. This includes the resource sharing option, and the setting to preempt low priority tasks.</p>
    pub fn set_compute_quota_config(mut self, input: ::std::option::Option<crate::types::ComputeQuotaConfig>) -> Self {
        self.inner = self.inner.set_compute_quota_config(input);
        self
    }
    /// <p>Configuration of the compute allocation definition. This includes the resource sharing option, and the setting to preempt low priority tasks.</p>
    pub fn get_compute_quota_config(&self) -> &::std::option::Option<crate::types::ComputeQuotaConfig> {
        self.inner.get_compute_quota_config()
    }
    /// <p>The target entity to allocate compute resources to.</p>
    pub fn compute_quota_target(mut self, input: crate::types::ComputeQuotaTarget) -> Self {
        self.inner = self.inner.compute_quota_target(input);
        self
    }
    /// <p>The target entity to allocate compute resources to.</p>
    pub fn set_compute_quota_target(mut self, input: ::std::option::Option<crate::types::ComputeQuotaTarget>) -> Self {
        self.inner = self.inner.set_compute_quota_target(input);
        self
    }
    /// <p>The target entity to allocate compute resources to.</p>
    pub fn get_compute_quota_target(&self) -> &::std::option::Option<crate::types::ComputeQuotaTarget> {
        self.inner.get_compute_quota_target()
    }
    /// <p>The state of the compute allocation being described. Use to enable or disable compute allocation.</p>
    /// <p>Default is <code>Enabled</code>.</p>
    pub fn activation_state(mut self, input: crate::types::ActivationState) -> Self {
        self.inner = self.inner.activation_state(input);
        self
    }
    /// <p>The state of the compute allocation being described. Use to enable or disable compute allocation.</p>
    /// <p>Default is <code>Enabled</code>.</p>
    pub fn set_activation_state(mut self, input: ::std::option::Option<crate::types::ActivationState>) -> Self {
        self.inner = self.inner.set_activation_state(input);
        self
    }
    /// <p>The state of the compute allocation being described. Use to enable or disable compute allocation.</p>
    /// <p>Default is <code>Enabled</code>.</p>
    pub fn get_activation_state(&self) -> &::std::option::Option<crate::types::ActivationState> {
        self.inner.get_activation_state()
    }
    /// <p>Description of the compute allocation definition.</p>
    pub fn description(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.description(input.into());
        self
    }
    /// <p>Description of the compute allocation definition.</p>
    pub fn set_description(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_description(input);
        self
    }
    /// <p>Description of the compute allocation definition.</p>
    pub fn get_description(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_description()
    }
}
