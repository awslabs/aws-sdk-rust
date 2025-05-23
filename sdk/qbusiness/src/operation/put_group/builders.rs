// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::put_group::_put_group_output::PutGroupOutputBuilder;

pub use crate::operation::put_group::_put_group_input::PutGroupInputBuilder;

impl crate::operation::put_group::builders::PutGroupInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::put_group::PutGroupOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::put_group::PutGroupError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.put_group();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `PutGroup`.
///
/// <p>Create, or updates, a mapping of users—who have access to a document—to groups.</p>
/// <p>You can also map sub groups to groups. For example, the group "Company Intellectual Property Teams" includes sub groups "Research" and "Engineering". These sub groups include their own list of users or people who work in these teams. Only users who work in research and engineering, and therefore belong in the intellectual property group, can see top-secret company documents in their Amazon Q Business chat results.</p>
/// <p>There are two options for creating groups, either passing group members inline or using an S3 file via the S3PathForGroupMembers field. For inline groups, there is a limit of 1000 members per group and for provided S3 files there is a limit of 100 thousand members. When creating a group using an S3 file, you provide both an S3 file and a <code>RoleArn</code> for Amazon Q Buisness to access the file.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct PutGroupFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::put_group::builders::PutGroupInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl crate::client::customize::internal::CustomizableSend<crate::operation::put_group::PutGroupOutput, crate::operation::put_group::PutGroupError>
    for PutGroupFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<crate::operation::put_group::PutGroupOutput, crate::operation::put_group::PutGroupError>,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl PutGroupFluentBuilder {
    /// Creates a new `PutGroupFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the PutGroup as a reference.
    pub fn as_input(&self) -> &crate::operation::put_group::builders::PutGroupInputBuilder {
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
        crate::operation::put_group::PutGroupOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::put_group::PutGroupError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::put_group::PutGroup::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::put_group::PutGroup::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<crate::operation::put_group::PutGroupOutput, crate::operation::put_group::PutGroupError, Self>
    {
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
    /// <p>The identifier of the application in which the user and group mapping belongs.</p>
    pub fn application_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.application_id(input.into());
        self
    }
    /// <p>The identifier of the application in which the user and group mapping belongs.</p>
    pub fn set_application_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_application_id(input);
        self
    }
    /// <p>The identifier of the application in which the user and group mapping belongs.</p>
    pub fn get_application_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_application_id()
    }
    /// <p>The identifier of the index in which you want to map users to their groups.</p>
    pub fn index_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.index_id(input.into());
        self
    }
    /// <p>The identifier of the index in which you want to map users to their groups.</p>
    pub fn set_index_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_index_id(input);
        self
    }
    /// <p>The identifier of the index in which you want to map users to their groups.</p>
    pub fn get_index_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_index_id()
    }
    /// <p>The list that contains your users or sub groups that belong the same group. For example, the group "Company" includes the user "CEO" and the sub groups "Research", "Engineering", and "Sales and Marketing".</p>
    pub fn group_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.group_name(input.into());
        self
    }
    /// <p>The list that contains your users or sub groups that belong the same group. For example, the group "Company" includes the user "CEO" and the sub groups "Research", "Engineering", and "Sales and Marketing".</p>
    pub fn set_group_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_group_name(input);
        self
    }
    /// <p>The list that contains your users or sub groups that belong the same group. For example, the group "Company" includes the user "CEO" and the sub groups "Research", "Engineering", and "Sales and Marketing".</p>
    pub fn get_group_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_group_name()
    }
    /// <p>The identifier of the data source for which you want to map users to their groups. This is useful if a group is tied to multiple data sources, but you only want the group to access documents of a certain data source. For example, the groups "Research", "Engineering", and "Sales and Marketing" are all tied to the company's documents stored in the data sources Confluence and Salesforce. However, "Sales and Marketing" team only needs access to customer-related documents stored in Salesforce.</p>
    pub fn data_source_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.data_source_id(input.into());
        self
    }
    /// <p>The identifier of the data source for which you want to map users to their groups. This is useful if a group is tied to multiple data sources, but you only want the group to access documents of a certain data source. For example, the groups "Research", "Engineering", and "Sales and Marketing" are all tied to the company's documents stored in the data sources Confluence and Salesforce. However, "Sales and Marketing" team only needs access to customer-related documents stored in Salesforce.</p>
    pub fn set_data_source_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_data_source_id(input);
        self
    }
    /// <p>The identifier of the data source for which you want to map users to their groups. This is useful if a group is tied to multiple data sources, but you only want the group to access documents of a certain data source. For example, the groups "Research", "Engineering", and "Sales and Marketing" are all tied to the company's documents stored in the data sources Confluence and Salesforce. However, "Sales and Marketing" team only needs access to customer-related documents stored in Salesforce.</p>
    pub fn get_data_source_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_data_source_id()
    }
    /// <p>The type of the group.</p>
    pub fn r#type(mut self, input: crate::types::MembershipType) -> Self {
        self.inner = self.inner.r#type(input);
        self
    }
    /// <p>The type of the group.</p>
    pub fn set_type(mut self, input: ::std::option::Option<crate::types::MembershipType>) -> Self {
        self.inner = self.inner.set_type(input);
        self
    }
    /// <p>The type of the group.</p>
    pub fn get_type(&self) -> &::std::option::Option<crate::types::MembershipType> {
        self.inner.get_type()
    }
    /// <p>A list of users or sub groups that belong to a group. This is for generating Amazon Q Business chat results only from document a user has access to.</p>
    pub fn group_members(mut self, input: crate::types::GroupMembers) -> Self {
        self.inner = self.inner.group_members(input);
        self
    }
    /// <p>A list of users or sub groups that belong to a group. This is for generating Amazon Q Business chat results only from document a user has access to.</p>
    pub fn set_group_members(mut self, input: ::std::option::Option<crate::types::GroupMembers>) -> Self {
        self.inner = self.inner.set_group_members(input);
        self
    }
    /// <p>A list of users or sub groups that belong to a group. This is for generating Amazon Q Business chat results only from document a user has access to.</p>
    pub fn get_group_members(&self) -> &::std::option::Option<crate::types::GroupMembers> {
        self.inner.get_group_members()
    }
    /// <p>The Amazon Resource Name (ARN) of an IAM role that has access to the S3 file that contains your list of users that belong to a group.</p>
    pub fn role_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.role_arn(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of an IAM role that has access to the S3 file that contains your list of users that belong to a group.</p>
    pub fn set_role_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_role_arn(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) of an IAM role that has access to the S3 file that contains your list of users that belong to a group.</p>
    pub fn get_role_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_role_arn()
    }
}
