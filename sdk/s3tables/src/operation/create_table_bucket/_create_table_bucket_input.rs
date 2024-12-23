// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct CreateTableBucketInput {
    /// <p>The name for the table bucket.</p>
    pub name: ::std::option::Option<::std::string::String>,
}
impl CreateTableBucketInput {
    /// <p>The name for the table bucket.</p>
    pub fn name(&self) -> ::std::option::Option<&str> {
        self.name.as_deref()
    }
}
impl CreateTableBucketInput {
    /// Creates a new builder-style object to manufacture [`CreateTableBucketInput`](crate::operation::create_table_bucket::CreateTableBucketInput).
    pub fn builder() -> crate::operation::create_table_bucket::builders::CreateTableBucketInputBuilder {
        crate::operation::create_table_bucket::builders::CreateTableBucketInputBuilder::default()
    }
}

/// A builder for [`CreateTableBucketInput`](crate::operation::create_table_bucket::CreateTableBucketInput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct CreateTableBucketInputBuilder {
    pub(crate) name: ::std::option::Option<::std::string::String>,
}
impl CreateTableBucketInputBuilder {
    /// <p>The name for the table bucket.</p>
    /// This field is required.
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.name = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The name for the table bucket.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.name = input;
        self
    }
    /// <p>The name for the table bucket.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.name
    }
    /// Consumes the builder and constructs a [`CreateTableBucketInput`](crate::operation::create_table_bucket::CreateTableBucketInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::operation::create_table_bucket::CreateTableBucketInput, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::operation::create_table_bucket::CreateTableBucketInput { name: self.name })
    }
}
