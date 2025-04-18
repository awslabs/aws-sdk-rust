// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::update_policy_store::_update_policy_store_output::UpdatePolicyStoreOutputBuilder;

pub use crate::operation::update_policy_store::_update_policy_store_input::UpdatePolicyStoreInputBuilder;

impl crate::operation::update_policy_store::builders::UpdatePolicyStoreInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::update_policy_store::UpdatePolicyStoreOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_policy_store::UpdatePolicyStoreError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.update_policy_store();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UpdatePolicyStore`.
///
/// <p>Modifies the validation setting for a policy store.</p><note>
/// <p>Verified Permissions is <i> <a href="https://wikipedia.org/wiki/Eventual_consistency">eventually consistent</a> </i>. It can take a few seconds for a new or changed element to propagate through the service and be visible in the results of other Verified Permissions operations.</p>
/// </note>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UpdatePolicyStoreFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::update_policy_store::builders::UpdatePolicyStoreInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::update_policy_store::UpdatePolicyStoreOutput,
        crate::operation::update_policy_store::UpdatePolicyStoreError,
    > for UpdatePolicyStoreFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::update_policy_store::UpdatePolicyStoreOutput,
            crate::operation::update_policy_store::UpdatePolicyStoreError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UpdatePolicyStoreFluentBuilder {
    /// Creates a new `UpdatePolicyStoreFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UpdatePolicyStore as a reference.
    pub fn as_input(&self) -> &crate::operation::update_policy_store::builders::UpdatePolicyStoreInputBuilder {
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
        crate::operation::update_policy_store::UpdatePolicyStoreOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_policy_store::UpdatePolicyStoreError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::update_policy_store::UpdatePolicyStore::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::update_policy_store::UpdatePolicyStore::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::update_policy_store::UpdatePolicyStoreOutput,
        crate::operation::update_policy_store::UpdatePolicyStoreError,
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
    /// <p>Specifies the ID of the policy store that you want to update</p>
    pub fn policy_store_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.policy_store_id(input.into());
        self
    }
    /// <p>Specifies the ID of the policy store that you want to update</p>
    pub fn set_policy_store_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_policy_store_id(input);
        self
    }
    /// <p>Specifies the ID of the policy store that you want to update</p>
    pub fn get_policy_store_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_policy_store_id()
    }
    /// <p>A structure that defines the validation settings that want to enable for the policy store.</p>
    pub fn validation_settings(mut self, input: crate::types::ValidationSettings) -> Self {
        self.inner = self.inner.validation_settings(input);
        self
    }
    /// <p>A structure that defines the validation settings that want to enable for the policy store.</p>
    pub fn set_validation_settings(mut self, input: ::std::option::Option<crate::types::ValidationSettings>) -> Self {
        self.inner = self.inner.set_validation_settings(input);
        self
    }
    /// <p>A structure that defines the validation settings that want to enable for the policy store.</p>
    pub fn get_validation_settings(&self) -> &::std::option::Option<crate::types::ValidationSettings> {
        self.inner.get_validation_settings()
    }
    /// <p>Specifies whether the policy store can be deleted. If enabled, the policy store can't be deleted.</p>
    /// <p>When you call <code>UpdatePolicyStore</code>, this parameter is unchanged unless explicitly included in the call.</p>
    pub fn deletion_protection(mut self, input: crate::types::DeletionProtection) -> Self {
        self.inner = self.inner.deletion_protection(input);
        self
    }
    /// <p>Specifies whether the policy store can be deleted. If enabled, the policy store can't be deleted.</p>
    /// <p>When you call <code>UpdatePolicyStore</code>, this parameter is unchanged unless explicitly included in the call.</p>
    pub fn set_deletion_protection(mut self, input: ::std::option::Option<crate::types::DeletionProtection>) -> Self {
        self.inner = self.inner.set_deletion_protection(input);
        self
    }
    /// <p>Specifies whether the policy store can be deleted. If enabled, the policy store can't be deleted.</p>
    /// <p>When you call <code>UpdatePolicyStore</code>, this parameter is unchanged unless explicitly included in the call.</p>
    pub fn get_deletion_protection(&self) -> &::std::option::Option<crate::types::DeletionProtection> {
        self.inner.get_deletion_protection()
    }
    /// <p>Descriptive text that you can provide to help with identification of the current policy store.</p>
    pub fn description(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.description(input.into());
        self
    }
    /// <p>Descriptive text that you can provide to help with identification of the current policy store.</p>
    pub fn set_description(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_description(input);
        self
    }
    /// <p>Descriptive text that you can provide to help with identification of the current policy store.</p>
    pub fn get_description(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_description()
    }
}
