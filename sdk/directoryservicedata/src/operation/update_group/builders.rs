// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::update_group::_update_group_output::UpdateGroupOutputBuilder;

pub use crate::operation::update_group::_update_group_input::UpdateGroupInputBuilder;

impl crate::operation::update_group::builders::UpdateGroupInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::update_group::UpdateGroupOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_group::UpdateGroupError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.update_group();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UpdateGroup`.
///
/// <p>Updates group information.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UpdateGroupFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::update_group::builders::UpdateGroupInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::update_group::UpdateGroupOutput,
        crate::operation::update_group::UpdateGroupError,
    > for UpdateGroupFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::update_group::UpdateGroupOutput,
            crate::operation::update_group::UpdateGroupError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UpdateGroupFluentBuilder {
    /// Creates a new `UpdateGroupFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UpdateGroup as a reference.
    pub fn as_input(&self) -> &crate::operation::update_group::builders::UpdateGroupInputBuilder {
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
        crate::operation::update_group::UpdateGroupOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_group::UpdateGroupError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::update_group::UpdateGroup::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::update_group::UpdateGroup::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::update_group::UpdateGroupOutput,
        crate::operation::update_group::UpdateGroupError,
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
    /// <p>The identifier (ID) of the directory that's associated with the group.</p>
    pub fn directory_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.directory_id(input.into());
        self
    }
    /// <p>The identifier (ID) of the directory that's associated with the group.</p>
    pub fn set_directory_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_directory_id(input);
        self
    }
    /// <p>The identifier (ID) of the directory that's associated with the group.</p>
    pub fn get_directory_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_directory_id()
    }
    /// <p>The name of the group.</p>
    pub fn sam_account_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.sam_account_name(input.into());
        self
    }
    /// <p>The name of the group.</p>
    pub fn set_sam_account_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_sam_account_name(input);
        self
    }
    /// <p>The name of the group.</p>
    pub fn get_sam_account_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_sam_account_name()
    }
    /// <p>The AD group type. For details, see <a href="https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-groups#how-active-directory-security-groups-work">Active Directory security group type</a>.</p>
    pub fn group_type(mut self, input: crate::types::GroupType) -> Self {
        self.inner = self.inner.group_type(input);
        self
    }
    /// <p>The AD group type. For details, see <a href="https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-groups#how-active-directory-security-groups-work">Active Directory security group type</a>.</p>
    pub fn set_group_type(mut self, input: ::std::option::Option<crate::types::GroupType>) -> Self {
        self.inner = self.inner.set_group_type(input);
        self
    }
    /// <p>The AD group type. For details, see <a href="https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-groups#how-active-directory-security-groups-work">Active Directory security group type</a>.</p>
    pub fn get_group_type(&self) -> &::std::option::Option<crate::types::GroupType> {
        self.inner.get_group_type()
    }
    /// <p>The scope of the AD group. For details, see <a href="https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-groups#group-scope">Active Directory security groups</a>.</p>
    pub fn group_scope(mut self, input: crate::types::GroupScope) -> Self {
        self.inner = self.inner.group_scope(input);
        self
    }
    /// <p>The scope of the AD group. For details, see <a href="https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-groups#group-scope">Active Directory security groups</a>.</p>
    pub fn set_group_scope(mut self, input: ::std::option::Option<crate::types::GroupScope>) -> Self {
        self.inner = self.inner.set_group_scope(input);
        self
    }
    /// <p>The scope of the AD group. For details, see <a href="https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-groups#group-scope">Active Directory security groups</a>.</p>
    pub fn get_group_scope(&self) -> &::std::option::Option<crate::types::GroupScope> {
        self.inner.get_group_scope()
    }
    ///
    /// Adds a key-value pair to `OtherAttributes`.
    ///
    /// To override the contents of this collection use [`set_other_attributes`](Self::set_other_attributes).
    ///
    /// <p>An expression that defines one or more attributes with the data type and the value of each attribute.</p>
    pub fn other_attributes(mut self, k: impl ::std::convert::Into<::std::string::String>, v: crate::types::AttributeValue) -> Self {
        self.inner = self.inner.other_attributes(k.into(), v);
        self
    }
    /// <p>An expression that defines one or more attributes with the data type and the value of each attribute.</p>
    pub fn set_other_attributes(
        mut self,
        input: ::std::option::Option<::std::collections::HashMap<::std::string::String, crate::types::AttributeValue>>,
    ) -> Self {
        self.inner = self.inner.set_other_attributes(input);
        self
    }
    /// <p>An expression that defines one or more attributes with the data type and the value of each attribute.</p>
    pub fn get_other_attributes(&self) -> &::std::option::Option<::std::collections::HashMap<::std::string::String, crate::types::AttributeValue>> {
        self.inner.get_other_attributes()
    }
    /// <p>The type of update to be performed. If no value exists for the attribute, use <code>ADD</code>. Otherwise, use <code>REPLACE</code> to change an attribute value or <code>REMOVE</code> to clear the attribute value.</p>
    pub fn update_type(mut self, input: crate::types::UpdateType) -> Self {
        self.inner = self.inner.update_type(input);
        self
    }
    /// <p>The type of update to be performed. If no value exists for the attribute, use <code>ADD</code>. Otherwise, use <code>REPLACE</code> to change an attribute value or <code>REMOVE</code> to clear the attribute value.</p>
    pub fn set_update_type(mut self, input: ::std::option::Option<crate::types::UpdateType>) -> Self {
        self.inner = self.inner.set_update_type(input);
        self
    }
    /// <p>The type of update to be performed. If no value exists for the attribute, use <code>ADD</code>. Otherwise, use <code>REPLACE</code> to change an attribute value or <code>REMOVE</code> to clear the attribute value.</p>
    pub fn get_update_type(&self) -> &::std::option::Option<crate::types::UpdateType> {
        self.inner.get_update_type()
    }
    /// <p>A unique and case-sensitive identifier that you provide to make sure the idempotency of the request, so multiple identical calls have the same effect as one single call.</p>
    /// <p>A client token is valid for 8 hours after the first request that uses it completes. After 8 hours, any request with the same client token is treated as a new request. If the request succeeds, any future uses of that token will be idempotent for another 8 hours.</p>
    /// <p>If you submit a request with the same client token but change one of the other parameters within the 8-hour idempotency window, Directory Service Data returns an <code>ConflictException</code>.</p><note>
    /// <p>This parameter is optional when using the CLI or SDK.</p>
    /// </note>
    pub fn client_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_token(input.into());
        self
    }
    /// <p>A unique and case-sensitive identifier that you provide to make sure the idempotency of the request, so multiple identical calls have the same effect as one single call.</p>
    /// <p>A client token is valid for 8 hours after the first request that uses it completes. After 8 hours, any request with the same client token is treated as a new request. If the request succeeds, any future uses of that token will be idempotent for another 8 hours.</p>
    /// <p>If you submit a request with the same client token but change one of the other parameters within the 8-hour idempotency window, Directory Service Data returns an <code>ConflictException</code>.</p><note>
    /// <p>This parameter is optional when using the CLI or SDK.</p>
    /// </note>
    pub fn set_client_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_token(input);
        self
    }
    /// <p>A unique and case-sensitive identifier that you provide to make sure the idempotency of the request, so multiple identical calls have the same effect as one single call.</p>
    /// <p>A client token is valid for 8 hours after the first request that uses it completes. After 8 hours, any request with the same client token is treated as a new request. If the request succeeds, any future uses of that token will be idempotent for another 8 hours.</p>
    /// <p>If you submit a request with the same client token but change one of the other parameters within the 8-hour idempotency window, Directory Service Data returns an <code>ConflictException</code>.</p><note>
    /// <p>This parameter is optional when using the CLI or SDK.</p>
    /// </note>
    pub fn get_client_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_token()
    }
}
