// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::describe_field_indexes::_describe_field_indexes_output::DescribeFieldIndexesOutputBuilder;

pub use crate::operation::describe_field_indexes::_describe_field_indexes_input::DescribeFieldIndexesInputBuilder;

impl crate::operation::describe_field_indexes::builders::DescribeFieldIndexesInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::describe_field_indexes::DescribeFieldIndexesOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_field_indexes::DescribeFieldIndexesError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.describe_field_indexes();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `DescribeFieldIndexes`.
///
/// <p>Returns a list of field indexes listed in the field index policies of one or more log groups. For more information about field index policies, see <a href="https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_PutIndexPolicy.html">PutIndexPolicy</a>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct DescribeFieldIndexesFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::describe_field_indexes::builders::DescribeFieldIndexesInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::describe_field_indexes::DescribeFieldIndexesOutput,
        crate::operation::describe_field_indexes::DescribeFieldIndexesError,
    > for DescribeFieldIndexesFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::describe_field_indexes::DescribeFieldIndexesOutput,
            crate::operation::describe_field_indexes::DescribeFieldIndexesError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl DescribeFieldIndexesFluentBuilder {
    /// Creates a new `DescribeFieldIndexesFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the DescribeFieldIndexes as a reference.
    pub fn as_input(&self) -> &crate::operation::describe_field_indexes::builders::DescribeFieldIndexesInputBuilder {
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
        crate::operation::describe_field_indexes::DescribeFieldIndexesOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_field_indexes::DescribeFieldIndexesError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::describe_field_indexes::DescribeFieldIndexes::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::describe_field_indexes::DescribeFieldIndexes::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::describe_field_indexes::DescribeFieldIndexesOutput,
        crate::operation::describe_field_indexes::DescribeFieldIndexesError,
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
    ///
    /// Appends an item to `logGroupIdentifiers`.
    ///
    /// To override the contents of this collection use [`set_log_group_identifiers`](Self::set_log_group_identifiers).
    ///
    /// <p>An array containing the names or ARNs of the log groups that you want to retrieve field indexes for.</p>
    pub fn log_group_identifiers(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.log_group_identifiers(input.into());
        self
    }
    /// <p>An array containing the names or ARNs of the log groups that you want to retrieve field indexes for.</p>
    pub fn set_log_group_identifiers(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_log_group_identifiers(input);
        self
    }
    /// <p>An array containing the names or ARNs of the log groups that you want to retrieve field indexes for.</p>
    pub fn get_log_group_identifiers(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_log_group_identifiers()
    }
    /// <p>The token for the next set of items to return. The token expires after 24 hours.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.next_token(input.into());
        self
    }
    /// <p>The token for the next set of items to return. The token expires after 24 hours.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_next_token(input);
        self
    }
    /// <p>The token for the next set of items to return. The token expires after 24 hours.</p>
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_next_token()
    }
}
