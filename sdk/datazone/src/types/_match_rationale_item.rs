// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>A rationale indicating why this item was matched by search.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub enum MatchRationaleItem {
    /// <p>A list of TextMatchItems.</p>
    TextMatches(::std::vec::Vec<crate::types::TextMatchItem>),
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
impl MatchRationaleItem {
    #[allow(irrefutable_let_patterns)]
    /// Tries to convert the enum instance into [`TextMatches`](crate::types::MatchRationaleItem::TextMatches), extracting the inner [`Vec`](::std::vec::Vec).
    /// Returns `Err(&Self)` if it can't be converted.
    pub fn as_text_matches(&self) -> ::std::result::Result<&::std::vec::Vec<crate::types::TextMatchItem>, &Self> {
        if let MatchRationaleItem::TextMatches(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }
    /// Returns true if this is a [`TextMatches`](crate::types::MatchRationaleItem::TextMatches).
    pub fn is_text_matches(&self) -> bool {
        self.as_text_matches().is_ok()
    }
    /// Returns true if the enum instance is the `Unknown` variant.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}
