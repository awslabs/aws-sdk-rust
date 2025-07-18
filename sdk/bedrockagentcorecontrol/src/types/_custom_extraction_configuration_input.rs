// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>Input for a custom extraction configuration.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub enum CustomExtractionConfigurationInput {
    /// <p>The semantic extraction override configuration input.</p>
    SemanticExtractionOverride(crate::types::SemanticOverrideExtractionConfigurationInput),
    /// <p>The user preference extraction override configuration input.</p>
    UserPreferenceExtractionOverride(crate::types::UserPreferenceOverrideExtractionConfigurationInput),
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
impl CustomExtractionConfigurationInput {
    /// Tries to convert the enum instance into [`SemanticExtractionOverride`](crate::types::CustomExtractionConfigurationInput::SemanticExtractionOverride), extracting the inner [`SemanticOverrideExtractionConfigurationInput`](crate::types::SemanticOverrideExtractionConfigurationInput).
    /// Returns `Err(&Self)` if it can't be converted.
    pub fn as_semantic_extraction_override(&self) -> ::std::result::Result<&crate::types::SemanticOverrideExtractionConfigurationInput, &Self> {
        if let CustomExtractionConfigurationInput::SemanticExtractionOverride(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }
    /// Returns true if this is a [`SemanticExtractionOverride`](crate::types::CustomExtractionConfigurationInput::SemanticExtractionOverride).
    pub fn is_semantic_extraction_override(&self) -> bool {
        self.as_semantic_extraction_override().is_ok()
    }
    /// Tries to convert the enum instance into [`UserPreferenceExtractionOverride`](crate::types::CustomExtractionConfigurationInput::UserPreferenceExtractionOverride), extracting the inner [`UserPreferenceOverrideExtractionConfigurationInput`](crate::types::UserPreferenceOverrideExtractionConfigurationInput).
    /// Returns `Err(&Self)` if it can't be converted.
    pub fn as_user_preference_extraction_override(
        &self,
    ) -> ::std::result::Result<&crate::types::UserPreferenceOverrideExtractionConfigurationInput, &Self> {
        if let CustomExtractionConfigurationInput::UserPreferenceExtractionOverride(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }
    /// Returns true if this is a [`UserPreferenceExtractionOverride`](crate::types::CustomExtractionConfigurationInput::UserPreferenceExtractionOverride).
    pub fn is_user_preference_extraction_override(&self) -> bool {
        self.as_user_preference_extraction_override().is_ok()
    }
    /// Returns true if the enum instance is the `Unknown` variant.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}
