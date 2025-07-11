// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct GetAccountActivityInput {
    /// <p>A unique identifier that identifies the activity.</p>
    pub activity_id: ::std::option::Option<::std::string::String>,
    /// <p>The language code used to return translated title and description fields.</p>
    pub language_code: ::std::option::Option<crate::types::LanguageCode>,
}
impl GetAccountActivityInput {
    /// <p>A unique identifier that identifies the activity.</p>
    pub fn activity_id(&self) -> ::std::option::Option<&str> {
        self.activity_id.as_deref()
    }
    /// <p>The language code used to return translated title and description fields.</p>
    pub fn language_code(&self) -> ::std::option::Option<&crate::types::LanguageCode> {
        self.language_code.as_ref()
    }
}
impl GetAccountActivityInput {
    /// Creates a new builder-style object to manufacture [`GetAccountActivityInput`](crate::operation::get_account_activity::GetAccountActivityInput).
    pub fn builder() -> crate::operation::get_account_activity::builders::GetAccountActivityInputBuilder {
        crate::operation::get_account_activity::builders::GetAccountActivityInputBuilder::default()
    }
}

/// A builder for [`GetAccountActivityInput`](crate::operation::get_account_activity::GetAccountActivityInput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct GetAccountActivityInputBuilder {
    pub(crate) activity_id: ::std::option::Option<::std::string::String>,
    pub(crate) language_code: ::std::option::Option<crate::types::LanguageCode>,
}
impl GetAccountActivityInputBuilder {
    /// <p>A unique identifier that identifies the activity.</p>
    /// This field is required.
    pub fn activity_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.activity_id = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>A unique identifier that identifies the activity.</p>
    pub fn set_activity_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.activity_id = input;
        self
    }
    /// <p>A unique identifier that identifies the activity.</p>
    pub fn get_activity_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.activity_id
    }
    /// <p>The language code used to return translated title and description fields.</p>
    pub fn language_code(mut self, input: crate::types::LanguageCode) -> Self {
        self.language_code = ::std::option::Option::Some(input);
        self
    }
    /// <p>The language code used to return translated title and description fields.</p>
    pub fn set_language_code(mut self, input: ::std::option::Option<crate::types::LanguageCode>) -> Self {
        self.language_code = input;
        self
    }
    /// <p>The language code used to return translated title and description fields.</p>
    pub fn get_language_code(&self) -> &::std::option::Option<crate::types::LanguageCode> {
        &self.language_code
    }
    /// Consumes the builder and constructs a [`GetAccountActivityInput`](crate::operation::get_account_activity::GetAccountActivityInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::operation::get_account_activity::GetAccountActivityInput, ::aws_smithy_types::error::operation::BuildError>
    {
        ::std::result::Result::Ok(crate::operation::get_account_activity::GetAccountActivityInput {
            activity_id: self.activity_id,
            language_code: self.language_code,
        })
    }
}
