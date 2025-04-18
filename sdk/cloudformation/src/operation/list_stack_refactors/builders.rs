// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::list_stack_refactors::_list_stack_refactors_output::ListStackRefactorsOutputBuilder;

pub use crate::operation::list_stack_refactors::_list_stack_refactors_input::ListStackRefactorsInputBuilder;

impl crate::operation::list_stack_refactors::builders::ListStackRefactorsInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::list_stack_refactors::ListStackRefactorsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::list_stack_refactors::ListStackRefactorsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.list_stack_refactors();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `ListStackRefactors`.
///
/// <p>Lists all account stack refactor operations and their statuses.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct ListStackRefactorsFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::list_stack_refactors::builders::ListStackRefactorsInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::list_stack_refactors::ListStackRefactorsOutput,
        crate::operation::list_stack_refactors::ListStackRefactorsError,
    > for ListStackRefactorsFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::list_stack_refactors::ListStackRefactorsOutput,
            crate::operation::list_stack_refactors::ListStackRefactorsError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl ListStackRefactorsFluentBuilder {
    /// Creates a new `ListStackRefactorsFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the ListStackRefactors as a reference.
    pub fn as_input(&self) -> &crate::operation::list_stack_refactors::builders::ListStackRefactorsInputBuilder {
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
        crate::operation::list_stack_refactors::ListStackRefactorsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::list_stack_refactors::ListStackRefactorsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::list_stack_refactors::ListStackRefactors::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::list_stack_refactors::ListStackRefactors::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::list_stack_refactors::ListStackRefactorsOutput,
        crate::operation::list_stack_refactors::ListStackRefactorsError,
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
    /// Paginators are used by calling [`send().await`](crate::operation::list_stack_refactors::paginator::ListStackRefactorsPaginator::send) which returns a [`PaginationStream`](aws_smithy_async::future::pagination_stream::PaginationStream).
    pub fn into_paginator(self) -> crate::operation::list_stack_refactors::paginator::ListStackRefactorsPaginator {
        crate::operation::list_stack_refactors::paginator::ListStackRefactorsPaginator::new(self.handle, self.inner)
    }
    ///
    /// Appends an item to `ExecutionStatusFilter`.
    ///
    /// To override the contents of this collection use [`set_execution_status_filter`](Self::set_execution_status_filter).
    ///
    /// <p>Execution status to use as a filter. Specify one or more execution status codes to list only stack refactors with the specified execution status codes.</p>
    pub fn execution_status_filter(mut self, input: crate::types::StackRefactorExecutionStatus) -> Self {
        self.inner = self.inner.execution_status_filter(input);
        self
    }
    /// <p>Execution status to use as a filter. Specify one or more execution status codes to list only stack refactors with the specified execution status codes.</p>
    pub fn set_execution_status_filter(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::StackRefactorExecutionStatus>>) -> Self {
        self.inner = self.inner.set_execution_status_filter(input);
        self
    }
    /// <p>Execution status to use as a filter. Specify one or more execution status codes to list only stack refactors with the specified execution status codes.</p>
    pub fn get_execution_status_filter(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::StackRefactorExecutionStatus>> {
        self.inner.get_execution_status_filter()
    }
    /// <p>If the request doesn't return all the remaining results, <code>NextToken</code> is set to a token. To retrieve the next set of results, call this action again and assign that token to the request object's <code>NextToken</code> parameter. If the request returns all results, <code>NextToken</code> is set to <code>null</code>.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.next_token(input.into());
        self
    }
    /// <p>If the request doesn't return all the remaining results, <code>NextToken</code> is set to a token. To retrieve the next set of results, call this action again and assign that token to the request object's <code>NextToken</code> parameter. If the request returns all results, <code>NextToken</code> is set to <code>null</code>.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_next_token(input);
        self
    }
    /// <p>If the request doesn't return all the remaining results, <code>NextToken</code> is set to a token. To retrieve the next set of results, call this action again and assign that token to the request object's <code>NextToken</code> parameter. If the request returns all results, <code>NextToken</code> is set to <code>null</code>.</p>
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_next_token()
    }
    /// <p>The maximum number of results to be returned with a single call. If the number of available results exceeds this maximum, the response includes a <code>NextToken</code> value that you can assign to the <code>NextToken</code> request parameter to get the next set of results.</p>
    pub fn max_results(mut self, input: i32) -> Self {
        self.inner = self.inner.max_results(input);
        self
    }
    /// <p>The maximum number of results to be returned with a single call. If the number of available results exceeds this maximum, the response includes a <code>NextToken</code> value that you can assign to the <code>NextToken</code> request parameter to get the next set of results.</p>
    pub fn set_max_results(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_max_results(input);
        self
    }
    /// <p>The maximum number of results to be returned with a single call. If the number of available results exceeds this maximum, the response includes a <code>NextToken</code> value that you can assign to the <code>NextToken</code> request parameter to get the next set of results.</p>
    pub fn get_max_results(&self) -> &::std::option::Option<i32> {
        self.inner.get_max_results()
    }
}
