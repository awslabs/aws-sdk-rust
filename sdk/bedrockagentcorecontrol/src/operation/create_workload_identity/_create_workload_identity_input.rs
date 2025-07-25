// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct CreateWorkloadIdentityInput {
    /// <p>The name of the workload identity. The name must be unique within your account.</p>
    pub name: ::std::option::Option<::std::string::String>,
    /// <p>The list of allowed OAuth2 return URLs for resources associated with this workload identity.</p>
    pub allowed_resource_oauth2_return_urls: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
}
impl CreateWorkloadIdentityInput {
    /// <p>The name of the workload identity. The name must be unique within your account.</p>
    pub fn name(&self) -> ::std::option::Option<&str> {
        self.name.as_deref()
    }
    /// <p>The list of allowed OAuth2 return URLs for resources associated with this workload identity.</p>
    ///
    /// If no value was sent for this field, a default will be set. If you want to determine if no value was sent, use `.allowed_resource_oauth2_return_urls.is_none()`.
    pub fn allowed_resource_oauth2_return_urls(&self) -> &[::std::string::String] {
        self.allowed_resource_oauth2_return_urls.as_deref().unwrap_or_default()
    }
}
impl CreateWorkloadIdentityInput {
    /// Creates a new builder-style object to manufacture [`CreateWorkloadIdentityInput`](crate::operation::create_workload_identity::CreateWorkloadIdentityInput).
    pub fn builder() -> crate::operation::create_workload_identity::builders::CreateWorkloadIdentityInputBuilder {
        crate::operation::create_workload_identity::builders::CreateWorkloadIdentityInputBuilder::default()
    }
}

/// A builder for [`CreateWorkloadIdentityInput`](crate::operation::create_workload_identity::CreateWorkloadIdentityInput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct CreateWorkloadIdentityInputBuilder {
    pub(crate) name: ::std::option::Option<::std::string::String>,
    pub(crate) allowed_resource_oauth2_return_urls: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
}
impl CreateWorkloadIdentityInputBuilder {
    /// <p>The name of the workload identity. The name must be unique within your account.</p>
    /// This field is required.
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.name = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The name of the workload identity. The name must be unique within your account.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.name = input;
        self
    }
    /// <p>The name of the workload identity. The name must be unique within your account.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.name
    }
    /// Appends an item to `allowed_resource_oauth2_return_urls`.
    ///
    /// To override the contents of this collection use [`set_allowed_resource_oauth2_return_urls`](Self::set_allowed_resource_oauth2_return_urls).
    ///
    /// <p>The list of allowed OAuth2 return URLs for resources associated with this workload identity.</p>
    pub fn allowed_resource_oauth2_return_urls(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        let mut v = self.allowed_resource_oauth2_return_urls.unwrap_or_default();
        v.push(input.into());
        self.allowed_resource_oauth2_return_urls = ::std::option::Option::Some(v);
        self
    }
    /// <p>The list of allowed OAuth2 return URLs for resources associated with this workload identity.</p>
    pub fn set_allowed_resource_oauth2_return_urls(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.allowed_resource_oauth2_return_urls = input;
        self
    }
    /// <p>The list of allowed OAuth2 return URLs for resources associated with this workload identity.</p>
    pub fn get_allowed_resource_oauth2_return_urls(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        &self.allowed_resource_oauth2_return_urls
    }
    /// Consumes the builder and constructs a [`CreateWorkloadIdentityInput`](crate::operation::create_workload_identity::CreateWorkloadIdentityInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::create_workload_identity::CreateWorkloadIdentityInput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::create_workload_identity::CreateWorkloadIdentityInput {
            name: self.name,
            allowed_resource_oauth2_return_urls: self.allowed_resource_oauth2_return_urls,
        })
    }
}
