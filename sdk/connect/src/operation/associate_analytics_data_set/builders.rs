// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::associate_analytics_data_set::_associate_analytics_data_set_output::AssociateAnalyticsDataSetOutputBuilder;

pub use crate::operation::associate_analytics_data_set::_associate_analytics_data_set_input::AssociateAnalyticsDataSetInputBuilder;

impl crate::operation::associate_analytics_data_set::builders::AssociateAnalyticsDataSetInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSetOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSetError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.associate_analytics_data_set();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `AssociateAnalyticsDataSet`.
///
/// <p>Associates the specified dataset for a Amazon Connect instance with the target account. You can associate only one dataset in a single call.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct AssociateAnalyticsDataSetFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::associate_analytics_data_set::builders::AssociateAnalyticsDataSetInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSetOutput,
        crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSetError,
    > for AssociateAnalyticsDataSetFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSetOutput,
            crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSetError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl AssociateAnalyticsDataSetFluentBuilder {
    /// Creates a new `AssociateAnalyticsDataSetFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the AssociateAnalyticsDataSet as a reference.
    pub fn as_input(&self) -> &crate::operation::associate_analytics_data_set::builders::AssociateAnalyticsDataSetInputBuilder {
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
        crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSetOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSetError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSet::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSet::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSetOutput,
        crate::operation::associate_analytics_data_set::AssociateAnalyticsDataSetError,
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
    /// <p>The identifier of the Amazon Connect instance. You can <a href="https://docs.aws.amazon.com/connect/latest/adminguide/find-instance-arn.html">find the instance ID</a> in the Amazon Resource Name (ARN) of the instance.</p>
    pub fn instance_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.instance_id(input.into());
        self
    }
    /// <p>The identifier of the Amazon Connect instance. You can <a href="https://docs.aws.amazon.com/connect/latest/adminguide/find-instance-arn.html">find the instance ID</a> in the Amazon Resource Name (ARN) of the instance.</p>
    pub fn set_instance_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_instance_id(input);
        self
    }
    /// <p>The identifier of the Amazon Connect instance. You can <a href="https://docs.aws.amazon.com/connect/latest/adminguide/find-instance-arn.html">find the instance ID</a> in the Amazon Resource Name (ARN) of the instance.</p>
    pub fn get_instance_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_instance_id()
    }
    /// <p>The identifier of the dataset to associate with the target account.</p>
    pub fn data_set_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.data_set_id(input.into());
        self
    }
    /// <p>The identifier of the dataset to associate with the target account.</p>
    pub fn set_data_set_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_data_set_id(input);
        self
    }
    /// <p>The identifier of the dataset to associate with the target account.</p>
    pub fn get_data_set_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_data_set_id()
    }
    /// <p>The identifier of the target account. Use to associate a dataset to a different account than the one containing the Amazon Connect instance. If not specified, by default this value is the Amazon Web Services account that has the Amazon Connect instance.</p>
    pub fn target_account_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.target_account_id(input.into());
        self
    }
    /// <p>The identifier of the target account. Use to associate a dataset to a different account than the one containing the Amazon Connect instance. If not specified, by default this value is the Amazon Web Services account that has the Amazon Connect instance.</p>
    pub fn set_target_account_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_target_account_id(input);
        self
    }
    /// <p>The identifier of the target account. Use to associate a dataset to a different account than the one containing the Amazon Connect instance. If not specified, by default this value is the Amazon Web Services account that has the Amazon Connect instance.</p>
    pub fn get_target_account_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_target_account_id()
    }
}
