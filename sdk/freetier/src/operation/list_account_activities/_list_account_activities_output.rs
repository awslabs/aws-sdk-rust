// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ListAccountActivitiesOutput {
    /// <p>A brief information about the activities.</p>
    pub activities: ::std::vec::Vec<crate::types::ActivitySummary>,
    /// <p>The token to include in another request to get the next page of items. This value is <code>null</code> when there are no more items to return.</p>
    pub next_token: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl ListAccountActivitiesOutput {
    /// <p>A brief information about the activities.</p>
    pub fn activities(&self) -> &[crate::types::ActivitySummary] {
        use std::ops::Deref;
        self.activities.deref()
    }
    /// <p>The token to include in another request to get the next page of items. This value is <code>null</code> when there are no more items to return.</p>
    pub fn next_token(&self) -> ::std::option::Option<&str> {
        self.next_token.as_deref()
    }
}
impl ::aws_types::request_id::RequestId for ListAccountActivitiesOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl ListAccountActivitiesOutput {
    /// Creates a new builder-style object to manufacture [`ListAccountActivitiesOutput`](crate::operation::list_account_activities::ListAccountActivitiesOutput).
    pub fn builder() -> crate::operation::list_account_activities::builders::ListAccountActivitiesOutputBuilder {
        crate::operation::list_account_activities::builders::ListAccountActivitiesOutputBuilder::default()
    }
}

/// A builder for [`ListAccountActivitiesOutput`](crate::operation::list_account_activities::ListAccountActivitiesOutput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct ListAccountActivitiesOutputBuilder {
    pub(crate) activities: ::std::option::Option<::std::vec::Vec<crate::types::ActivitySummary>>,
    pub(crate) next_token: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl ListAccountActivitiesOutputBuilder {
    /// Appends an item to `activities`.
    ///
    /// To override the contents of this collection use [`set_activities`](Self::set_activities).
    ///
    /// <p>A brief information about the activities.</p>
    pub fn activities(mut self, input: crate::types::ActivitySummary) -> Self {
        let mut v = self.activities.unwrap_or_default();
        v.push(input);
        self.activities = ::std::option::Option::Some(v);
        self
    }
    /// <p>A brief information about the activities.</p>
    pub fn set_activities(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::ActivitySummary>>) -> Self {
        self.activities = input;
        self
    }
    /// <p>A brief information about the activities.</p>
    pub fn get_activities(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::ActivitySummary>> {
        &self.activities
    }
    /// <p>The token to include in another request to get the next page of items. This value is <code>null</code> when there are no more items to return.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.next_token = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The token to include in another request to get the next page of items. This value is <code>null</code> when there are no more items to return.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.next_token = input;
        self
    }
    /// <p>The token to include in another request to get the next page of items. This value is <code>null</code> when there are no more items to return.</p>
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
    /// Consumes the builder and constructs a [`ListAccountActivitiesOutput`](crate::operation::list_account_activities::ListAccountActivitiesOutput).
    /// This method will fail if any of the following fields are not set:
    /// - [`activities`](crate::operation::list_account_activities::builders::ListAccountActivitiesOutputBuilder::activities)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::operation::list_account_activities::ListAccountActivitiesOutput, ::aws_smithy_types::error::operation::BuildError>
    {
        ::std::result::Result::Ok(crate::operation::list_account_activities::ListAccountActivitiesOutput {
            activities: self.activities.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "activities",
                    "activities was not specified but it is required when building ListAccountActivitiesOutput",
                )
            })?,
            next_token: self.next_token,
            _request_id: self._request_id,
        })
    }
}
