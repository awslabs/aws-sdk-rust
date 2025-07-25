// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ListAgentRuntimesOutput {
    /// <p>The list of agent runtimes.</p>
    pub agent_runtimes: ::std::vec::Vec<crate::types::Agent>,
    /// <p>A token to retrieve the next page of results.</p>
    pub next_token: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl ListAgentRuntimesOutput {
    /// <p>The list of agent runtimes.</p>
    pub fn agent_runtimes(&self) -> &[crate::types::Agent] {
        use std::ops::Deref;
        self.agent_runtimes.deref()
    }
    /// <p>A token to retrieve the next page of results.</p>
    pub fn next_token(&self) -> ::std::option::Option<&str> {
        self.next_token.as_deref()
    }
}
impl ::aws_types::request_id::RequestId for ListAgentRuntimesOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl ListAgentRuntimesOutput {
    /// Creates a new builder-style object to manufacture [`ListAgentRuntimesOutput`](crate::operation::list_agent_runtimes::ListAgentRuntimesOutput).
    pub fn builder() -> crate::operation::list_agent_runtimes::builders::ListAgentRuntimesOutputBuilder {
        crate::operation::list_agent_runtimes::builders::ListAgentRuntimesOutputBuilder::default()
    }
}

/// A builder for [`ListAgentRuntimesOutput`](crate::operation::list_agent_runtimes::ListAgentRuntimesOutput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct ListAgentRuntimesOutputBuilder {
    pub(crate) agent_runtimes: ::std::option::Option<::std::vec::Vec<crate::types::Agent>>,
    pub(crate) next_token: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl ListAgentRuntimesOutputBuilder {
    /// Appends an item to `agent_runtimes`.
    ///
    /// To override the contents of this collection use [`set_agent_runtimes`](Self::set_agent_runtimes).
    ///
    /// <p>The list of agent runtimes.</p>
    pub fn agent_runtimes(mut self, input: crate::types::Agent) -> Self {
        let mut v = self.agent_runtimes.unwrap_or_default();
        v.push(input);
        self.agent_runtimes = ::std::option::Option::Some(v);
        self
    }
    /// <p>The list of agent runtimes.</p>
    pub fn set_agent_runtimes(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Agent>>) -> Self {
        self.agent_runtimes = input;
        self
    }
    /// <p>The list of agent runtimes.</p>
    pub fn get_agent_runtimes(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Agent>> {
        &self.agent_runtimes
    }
    /// <p>A token to retrieve the next page of results.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.next_token = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>A token to retrieve the next page of results.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.next_token = input;
        self
    }
    /// <p>A token to retrieve the next page of results.</p>
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        &self.next_token
    }
    pub(crate) fn _request_id(mut self, request_id: impl Into<String>) -> Self {
        self._request_id = Some(request_id.into());
        self
    }

    pub(crate) fn _set_request_id(&mut self, request_id: Option<String>) -> &mut Self {
        self._request_id = request_id;
        self
    }
    /// Consumes the builder and constructs a [`ListAgentRuntimesOutput`](crate::operation::list_agent_runtimes::ListAgentRuntimesOutput).
    /// This method will fail if any of the following fields are not set:
    /// - [`agent_runtimes`](crate::operation::list_agent_runtimes::builders::ListAgentRuntimesOutputBuilder::agent_runtimes)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::operation::list_agent_runtimes::ListAgentRuntimesOutput, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::operation::list_agent_runtimes::ListAgentRuntimesOutput {
            agent_runtimes: self.agent_runtimes.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "agent_runtimes",
                    "agent_runtimes was not specified but it is required when building ListAgentRuntimesOutput",
                )
            })?,
            next_token: self.next_token,
            _request_id: self._request_id,
        })
    }
}
