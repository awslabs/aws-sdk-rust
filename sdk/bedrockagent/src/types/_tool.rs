// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>Contains configurations for a tool that a model can use when generating a response. For more information, see <a href="https://docs.aws.amazon.com/bedrock/latest/userguide/tool-use.html">Use a tool to complete an Amazon Bedrock model response</a>.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub enum Tool {
    /// <p>The specification for the tool.</p>
    ToolSpec(crate::types::ToolSpecification),
    /// The `Unknown` variant represents cases where new union variant was received. Consider upgrading the SDK to the latest available version.
    /// An unknown enum variant
    ///
    /// _Note: If you encounter this error, consider upgrading your SDK to the latest version._
    /// The `Unknown` variant represents cases where the server sent a value that wasn't recognized
    /// by the client. This can happen when the server adds new functionality, but the client has not been updated.
    /// To investigate this, consider turning on debug logging to print the raw HTTP response.
    #[non_exhaustive]
    Unknown,
}
impl Tool {
    #[allow(irrefutable_let_patterns)]
    /// Tries to convert the enum instance into [`ToolSpec`](crate::types::Tool::ToolSpec), extracting the inner [`ToolSpecification`](crate::types::ToolSpecification).
    /// Returns `Err(&Self)` if it can't be converted.
    pub fn as_tool_spec(&self) -> ::std::result::Result<&crate::types::ToolSpecification, &Self> {
        if let Tool::ToolSpec(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }
    /// Returns true if this is a [`ToolSpec`](crate::types::Tool::ToolSpec).
    pub fn is_tool_spec(&self) -> bool {
        self.as_tool_spec().is_ok()
    }
    /// Returns true if the enum instance is the `Unknown` variant.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}
