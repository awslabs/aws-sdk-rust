// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::describe_stack_events::_describe_stack_events_output::DescribeStackEventsOutputBuilder;

pub use crate::operation::describe_stack_events::_describe_stack_events_input::DescribeStackEventsInputBuilder;

impl crate::operation::describe_stack_events::builders::DescribeStackEventsInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::describe_stack_events::DescribeStackEventsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_stack_events::DescribeStackEventsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.describe_stack_events();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `DescribeStackEvents`.
///
/// <p>Returns all stack related events for a specified stack in reverse chronological order. For more information about a stack's event history, see <a href="https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/stack-resource-configuration-complete.html">Understand CloudFormation stack creation events</a> in the <i>CloudFormation User Guide</i>.</p><note>
/// <p>You can list events for stacks that have failed to create or have been deleted by specifying the unique stack identifier (stack ID).</p>
/// </note>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct DescribeStackEventsFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::describe_stack_events::builders::DescribeStackEventsInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::describe_stack_events::DescribeStackEventsOutput,
        crate::operation::describe_stack_events::DescribeStackEventsError,
    > for DescribeStackEventsFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::describe_stack_events::DescribeStackEventsOutput,
            crate::operation::describe_stack_events::DescribeStackEventsError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl DescribeStackEventsFluentBuilder {
    /// Creates a new `DescribeStackEventsFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the DescribeStackEvents as a reference.
    pub fn as_input(&self) -> &crate::operation::describe_stack_events::builders::DescribeStackEventsInputBuilder {
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
        crate::operation::describe_stack_events::DescribeStackEventsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_stack_events::DescribeStackEventsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::describe_stack_events::DescribeStackEvents::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::describe_stack_events::DescribeStackEvents::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::describe_stack_events::DescribeStackEventsOutput,
        crate::operation::describe_stack_events::DescribeStackEventsError,
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
    /// Paginators are used by calling [`send().await`](crate::operation::describe_stack_events::paginator::DescribeStackEventsPaginator::send) which returns a [`PaginationStream`](aws_smithy_async::future::pagination_stream::PaginationStream).
    pub fn into_paginator(self) -> crate::operation::describe_stack_events::paginator::DescribeStackEventsPaginator {
        crate::operation::describe_stack_events::paginator::DescribeStackEventsPaginator::new(self.handle, self.inner)
    }
    /// <p>The name or the unique stack ID that's associated with the stack, which aren't always interchangeable:</p>
    /// <ul>
    /// <li>
    /// <p>Running stacks: You can specify either the stack's name or its unique stack ID.</p></li>
    /// <li>
    /// <p>Deleted stacks: You must specify the unique stack ID.</p></li>
    /// </ul>
    pub fn stack_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.stack_name(input.into());
        self
    }
    /// <p>The name or the unique stack ID that's associated with the stack, which aren't always interchangeable:</p>
    /// <ul>
    /// <li>
    /// <p>Running stacks: You can specify either the stack's name or its unique stack ID.</p></li>
    /// <li>
    /// <p>Deleted stacks: You must specify the unique stack ID.</p></li>
    /// </ul>
    pub fn set_stack_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_stack_name(input);
        self
    }
    /// <p>The name or the unique stack ID that's associated with the stack, which aren't always interchangeable:</p>
    /// <ul>
    /// <li>
    /// <p>Running stacks: You can specify either the stack's name or its unique stack ID.</p></li>
    /// <li>
    /// <p>Deleted stacks: You must specify the unique stack ID.</p></li>
    /// </ul>
    pub fn get_stack_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_stack_name()
    }
    /// <p>A string that identifies the next page of events that you want to retrieve.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.next_token(input.into());
        self
    }
    /// <p>A string that identifies the next page of events that you want to retrieve.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_next_token(input);
        self
    }
    /// <p>A string that identifies the next page of events that you want to retrieve.</p>
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_next_token()
    }
}
