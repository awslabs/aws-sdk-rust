// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ListChatResponseConfigurationsInput {
    /// <p>The unique identifier of the Amazon Q Business application for which to list available chat response configurations.</p>
    pub application_id: ::std::option::Option<::std::string::String>,
    /// <p>The maximum number of chat response configurations to return in a single response. This parameter helps control pagination of results when many configurations exist.</p>
    pub max_results: ::std::option::Option<i32>,
    /// <p>A pagination token used to retrieve the next set of results when the number of configurations exceeds the specified <code>maxResults</code> value.</p>
    pub next_token: ::std::option::Option<::std::string::String>,
}
impl ListChatResponseConfigurationsInput {
    /// <p>The unique identifier of the Amazon Q Business application for which to list available chat response configurations.</p>
    pub fn application_id(&self) -> ::std::option::Option<&str> {
        self.application_id.as_deref()
    }
    /// <p>The maximum number of chat response configurations to return in a single response. This parameter helps control pagination of results when many configurations exist.</p>
    pub fn max_results(&self) -> ::std::option::Option<i32> {
        self.max_results
    }
    /// <p>A pagination token used to retrieve the next set of results when the number of configurations exceeds the specified <code>maxResults</code> value.</p>
    pub fn next_token(&self) -> ::std::option::Option<&str> {
        self.next_token.as_deref()
    }
}
impl ListChatResponseConfigurationsInput {
    /// Creates a new builder-style object to manufacture [`ListChatResponseConfigurationsInput`](crate::operation::list_chat_response_configurations::ListChatResponseConfigurationsInput).
    pub fn builder() -> crate::operation::list_chat_response_configurations::builders::ListChatResponseConfigurationsInputBuilder {
        crate::operation::list_chat_response_configurations::builders::ListChatResponseConfigurationsInputBuilder::default()
    }
}

/// A builder for [`ListChatResponseConfigurationsInput`](crate::operation::list_chat_response_configurations::ListChatResponseConfigurationsInput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct ListChatResponseConfigurationsInputBuilder {
    pub(crate) application_id: ::std::option::Option<::std::string::String>,
    pub(crate) max_results: ::std::option::Option<i32>,
    pub(crate) next_token: ::std::option::Option<::std::string::String>,
}
impl ListChatResponseConfigurationsInputBuilder {
    /// <p>The unique identifier of the Amazon Q Business application for which to list available chat response configurations.</p>
    /// This field is required.
    pub fn application_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.application_id = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The unique identifier of the Amazon Q Business application for which to list available chat response configurations.</p>
    pub fn set_application_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.application_id = input;
        self
    }
    /// <p>The unique identifier of the Amazon Q Business application for which to list available chat response configurations.</p>
    pub fn get_application_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.application_id
    }
    /// <p>The maximum number of chat response configurations to return in a single response. This parameter helps control pagination of results when many configurations exist.</p>
    pub fn max_results(mut self, input: i32) -> Self {
        self.max_results = ::std::option::Option::Some(input);
        self
    }
    /// <p>The maximum number of chat response configurations to return in a single response. This parameter helps control pagination of results when many configurations exist.</p>
    pub fn set_max_results(mut self, input: ::std::option::Option<i32>) -> Self {
        self.max_results = input;
        self
    }
    /// <p>The maximum number of chat response configurations to return in a single response. This parameter helps control pagination of results when many configurations exist.</p>
    pub fn get_max_results(&self) -> &::std::option::Option<i32> {
        &self.max_results
    }
    /// <p>A pagination token used to retrieve the next set of results when the number of configurations exceeds the specified <code>maxResults</code> value.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.next_token = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>A pagination token used to retrieve the next set of results when the number of configurations exceeds the specified <code>maxResults</code> value.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.next_token = input;
        self
    }
    /// <p>A pagination token used to retrieve the next set of results when the number of configurations exceeds the specified <code>maxResults</code> value.</p>
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        &self.next_token
    }
    /// Consumes the builder and constructs a [`ListChatResponseConfigurationsInput`](crate::operation::list_chat_response_configurations::ListChatResponseConfigurationsInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::list_chat_response_configurations::ListChatResponseConfigurationsInput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::list_chat_response_configurations::ListChatResponseConfigurationsInput {
            application_id: self.application_id,
            max_results: self.max_results,
            next_token: self.next_token,
        })
    }
}
