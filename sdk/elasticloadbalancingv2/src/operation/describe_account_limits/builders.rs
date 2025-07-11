// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::describe_account_limits::_describe_account_limits_output::DescribeAccountLimitsOutputBuilder;

pub use crate::operation::describe_account_limits::_describe_account_limits_input::DescribeAccountLimitsInputBuilder;

impl crate::operation::describe_account_limits::builders::DescribeAccountLimitsInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::describe_account_limits::DescribeAccountLimitsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_account_limits::DescribeAccountLimitsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.describe_account_limits();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `DescribeAccountLimits`.
///
/// <p>Describes the current Elastic Load Balancing resource limits for your Amazon Web Services account.</p>
/// <p>For more information, see the following:</p>
/// <ul>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/elasticloadbalancing/latest/application/load-balancer-limits.html">Quotas for your Application Load Balancers</a></p></li>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/elasticloadbalancing/latest/network/load-balancer-limits.html">Quotas for your Network Load Balancers</a></p></li>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/elasticloadbalancing/latest/gateway/quotas-limits.html">Quotas for your Gateway Load Balancers</a></p></li>
/// </ul>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct DescribeAccountLimitsFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::describe_account_limits::builders::DescribeAccountLimitsInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::describe_account_limits::DescribeAccountLimitsOutput,
        crate::operation::describe_account_limits::DescribeAccountLimitsError,
    > for DescribeAccountLimitsFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::describe_account_limits::DescribeAccountLimitsOutput,
            crate::operation::describe_account_limits::DescribeAccountLimitsError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl DescribeAccountLimitsFluentBuilder {
    /// Creates a new `DescribeAccountLimitsFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the DescribeAccountLimits as a reference.
    pub fn as_input(&self) -> &crate::operation::describe_account_limits::builders::DescribeAccountLimitsInputBuilder {
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
        crate::operation::describe_account_limits::DescribeAccountLimitsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_account_limits::DescribeAccountLimitsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::describe_account_limits::DescribeAccountLimits::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::describe_account_limits::DescribeAccountLimits::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::describe_account_limits::DescribeAccountLimitsOutput,
        crate::operation::describe_account_limits::DescribeAccountLimitsError,
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
    /// Create a paginator for this request
    ///
    /// Paginators are used by calling [`send().await`](crate::operation::describe_account_limits::paginator::DescribeAccountLimitsPaginator::send) which returns a [`PaginationStream`](aws_smithy_async::future::pagination_stream::PaginationStream).
    pub fn into_paginator(self) -> crate::operation::describe_account_limits::paginator::DescribeAccountLimitsPaginator {
        crate::operation::describe_account_limits::paginator::DescribeAccountLimitsPaginator::new(self.handle, self.inner)
    }
    /// <p>The marker for the next set of results. (You received this marker from a previous call.)</p>
    pub fn marker(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.marker(input.into());
        self
    }
    /// <p>The marker for the next set of results. (You received this marker from a previous call.)</p>
    pub fn set_marker(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_marker(input);
        self
    }
    /// <p>The marker for the next set of results. (You received this marker from a previous call.)</p>
    pub fn get_marker(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_marker()
    }
    /// <p>The maximum number of results to return with this call.</p>
    pub fn page_size(mut self, input: i32) -> Self {
        self.inner = self.inner.page_size(input);
        self
    }
    /// <p>The maximum number of results to return with this call.</p>
    pub fn set_page_size(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_page_size(input);
        self
    }
    /// <p>The maximum number of results to return with this call.</p>
    pub fn get_page_size(&self) -> &::std::option::Option<i32> {
        self.inner.get_page_size()
    }
}
