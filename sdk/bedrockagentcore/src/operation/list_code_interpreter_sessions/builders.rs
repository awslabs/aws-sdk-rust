// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::list_code_interpreter_sessions::_list_code_interpreter_sessions_output::ListCodeInterpreterSessionsOutputBuilder;

pub use crate::operation::list_code_interpreter_sessions::_list_code_interpreter_sessions_input::ListCodeInterpreterSessionsInputBuilder;

impl crate::operation::list_code_interpreter_sessions::builders::ListCodeInterpreterSessionsInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessionsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessionsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.list_code_interpreter_sessions();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `ListCodeInterpreterSessions`.
///
/// <p>Retrieves a list of code interpreter sessions in Amazon Bedrock that match the specified criteria. This operation returns summary information about each session, including identifiers, status, and timestamps.</p>
/// <p>You can filter the results by code interpreter identifier and session status. The operation supports pagination to handle large result sets efficiently.</p>
/// <p>We recommend using pagination to ensure that the operation returns quickly and successfully when retrieving large numbers of sessions.</p>
/// <p>The following operations are related to <code>ListCodeInterpreterSessions</code>:</p>
/// <ul>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/API_StartCodeInterpreterSession.html">StartCodeInterpreterSession</a></p></li>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/API_GetCodeInterpreterSession.html">GetCodeInterpreterSession</a></p></li>
/// </ul>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct ListCodeInterpreterSessionsFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::list_code_interpreter_sessions::builders::ListCodeInterpreterSessionsInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessionsOutput,
        crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessionsError,
    > for ListCodeInterpreterSessionsFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessionsOutput,
            crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessionsError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl ListCodeInterpreterSessionsFluentBuilder {
    /// Creates a new `ListCodeInterpreterSessionsFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the ListCodeInterpreterSessions as a reference.
    pub fn as_input(&self) -> &crate::operation::list_code_interpreter_sessions::builders::ListCodeInterpreterSessionsInputBuilder {
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
        crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessionsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessionsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessions::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessions::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessionsOutput,
        crate::operation::list_code_interpreter_sessions::ListCodeInterpreterSessionsError,
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
    /// <p>The unique identifier of the code interpreter to list sessions for. If specified, only sessions for this code interpreter are returned. If not specified, sessions for all code interpreters are returned.</p>
    pub fn code_interpreter_identifier(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.code_interpreter_identifier(input.into());
        self
    }
    /// <p>The unique identifier of the code interpreter to list sessions for. If specified, only sessions for this code interpreter are returned. If not specified, sessions for all code interpreters are returned.</p>
    pub fn set_code_interpreter_identifier(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_code_interpreter_identifier(input);
        self
    }
    /// <p>The unique identifier of the code interpreter to list sessions for. If specified, only sessions for this code interpreter are returned. If not specified, sessions for all code interpreters are returned.</p>
    pub fn get_code_interpreter_identifier(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_code_interpreter_identifier()
    }
    /// <p>The maximum number of results to return in a single call. The default value is 10. Valid values range from 1 to 100. To retrieve the remaining results, make another call with the returned <code>nextToken</code> value.</p>
    pub fn max_results(mut self, input: i32) -> Self {
        self.inner = self.inner.max_results(input);
        self
    }
    /// <p>The maximum number of results to return in a single call. The default value is 10. Valid values range from 1 to 100. To retrieve the remaining results, make another call with the returned <code>nextToken</code> value.</p>
    pub fn set_max_results(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_max_results(input);
        self
    }
    /// <p>The maximum number of results to return in a single call. The default value is 10. Valid values range from 1 to 100. To retrieve the remaining results, make another call with the returned <code>nextToken</code> value.</p>
    pub fn get_max_results(&self) -> &::std::option::Option<i32> {
        self.inner.get_max_results()
    }
    /// <p>The token for the next set of results. Use the value returned in the previous response in the next request to retrieve the next set of results. If not specified, Amazon Bedrock returns the first page of results.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.next_token(input.into());
        self
    }
    /// <p>The token for the next set of results. Use the value returned in the previous response in the next request to retrieve the next set of results. If not specified, Amazon Bedrock returns the first page of results.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_next_token(input);
        self
    }
    /// <p>The token for the next set of results. Use the value returned in the previous response in the next request to retrieve the next set of results. If not specified, Amazon Bedrock returns the first page of results.</p>
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_next_token()
    }
    /// <p>The status of the code interpreter sessions to list. Valid values include ACTIVE, STOPPING, and STOPPED. If not specified, sessions with any status are returned.</p>
    pub fn status(mut self, input: crate::types::CodeInterpreterSessionStatus) -> Self {
        self.inner = self.inner.status(input);
        self
    }
    /// <p>The status of the code interpreter sessions to list. Valid values include ACTIVE, STOPPING, and STOPPED. If not specified, sessions with any status are returned.</p>
    pub fn set_status(mut self, input: ::std::option::Option<crate::types::CodeInterpreterSessionStatus>) -> Self {
        self.inner = self.inner.set_status(input);
        self
    }
    /// <p>The status of the code interpreter sessions to list. Valid values include ACTIVE, STOPPING, and STOPPED. If not specified, sessions with any status are returned.</p>
    pub fn get_status(&self) -> &::std::option::Option<crate::types::CodeInterpreterSessionStatus> {
        self.inner.get_status()
    }
}
