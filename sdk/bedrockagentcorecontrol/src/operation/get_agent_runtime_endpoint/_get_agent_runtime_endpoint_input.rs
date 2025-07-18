// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq)]
pub struct GetAgentRuntimeEndpointInput {
    /// <p>The unique identifier of the agent runtime associated with the endpoint.</p>
    pub agent_runtime_id: ::std::option::Option<::std::string::String>,
    /// <p>The name of the agent runtime endpoint to retrieve.</p>
    pub endpoint_name: ::std::option::Option<::std::string::String>,
}
impl GetAgentRuntimeEndpointInput {
    /// <p>The unique identifier of the agent runtime associated with the endpoint.</p>
    pub fn agent_runtime_id(&self) -> ::std::option::Option<&str> {
        self.agent_runtime_id.as_deref()
    }
    /// <p>The name of the agent runtime endpoint to retrieve.</p>
    pub fn endpoint_name(&self) -> ::std::option::Option<&str> {
        self.endpoint_name.as_deref()
    }
}
impl ::std::fmt::Debug for GetAgentRuntimeEndpointInput {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("GetAgentRuntimeEndpointInput");
        formatter.field("agent_runtime_id", &self.agent_runtime_id);
        formatter.field("endpoint_name", &"*** Sensitive Data Redacted ***");
        formatter.finish()
    }
}
impl GetAgentRuntimeEndpointInput {
    /// Creates a new builder-style object to manufacture [`GetAgentRuntimeEndpointInput`](crate::operation::get_agent_runtime_endpoint::GetAgentRuntimeEndpointInput).
    pub fn builder() -> crate::operation::get_agent_runtime_endpoint::builders::GetAgentRuntimeEndpointInputBuilder {
        crate::operation::get_agent_runtime_endpoint::builders::GetAgentRuntimeEndpointInputBuilder::default()
    }
}

/// A builder for [`GetAgentRuntimeEndpointInput`](crate::operation::get_agent_runtime_endpoint::GetAgentRuntimeEndpointInput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default)]
#[non_exhaustive]
pub struct GetAgentRuntimeEndpointInputBuilder {
    pub(crate) agent_runtime_id: ::std::option::Option<::std::string::String>,
    pub(crate) endpoint_name: ::std::option::Option<::std::string::String>,
}
impl GetAgentRuntimeEndpointInputBuilder {
    /// <p>The unique identifier of the agent runtime associated with the endpoint.</p>
    /// This field is required.
    pub fn agent_runtime_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.agent_runtime_id = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The unique identifier of the agent runtime associated with the endpoint.</p>
    pub fn set_agent_runtime_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.agent_runtime_id = input;
        self
    }
    /// <p>The unique identifier of the agent runtime associated with the endpoint.</p>
    pub fn get_agent_runtime_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.agent_runtime_id
    }
    /// <p>The name of the agent runtime endpoint to retrieve.</p>
    /// This field is required.
    pub fn endpoint_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.endpoint_name = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The name of the agent runtime endpoint to retrieve.</p>
    pub fn set_endpoint_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.endpoint_name = input;
        self
    }
    /// <p>The name of the agent runtime endpoint to retrieve.</p>
    pub fn get_endpoint_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.endpoint_name
    }
    /// Consumes the builder and constructs a [`GetAgentRuntimeEndpointInput`](crate::operation::get_agent_runtime_endpoint::GetAgentRuntimeEndpointInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::get_agent_runtime_endpoint::GetAgentRuntimeEndpointInput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::get_agent_runtime_endpoint::GetAgentRuntimeEndpointInput {
            agent_runtime_id: self.agent_runtime_id,
            endpoint_name: self.endpoint_name,
        })
    }
}
impl ::std::fmt::Debug for GetAgentRuntimeEndpointInputBuilder {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut formatter = f.debug_struct("GetAgentRuntimeEndpointInputBuilder");
        formatter.field("agent_runtime_id", &self.agent_runtime_id);
        formatter.field("endpoint_name", &"*** Sensitive Data Redacted ***");
        formatter.finish()
    }
}
