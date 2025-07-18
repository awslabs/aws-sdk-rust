// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>The Amazon S3 location for storing data. This structure defines where in Amazon S3 data is stored.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct S3Location {
    /// <p>The name of the Amazon S3 bucket. This bucket contains the stored data.</p>
    pub bucket: ::std::string::String,
    /// <p>The prefix for objects in the Amazon S3 bucket. This prefix is added to the object keys to organize the data.</p>
    pub prefix: ::std::string::String,
}
impl S3Location {
    /// <p>The name of the Amazon S3 bucket. This bucket contains the stored data.</p>
    pub fn bucket(&self) -> &str {
        use std::ops::Deref;
        self.bucket.deref()
    }
    /// <p>The prefix for objects in the Amazon S3 bucket. This prefix is added to the object keys to organize the data.</p>
    pub fn prefix(&self) -> &str {
        use std::ops::Deref;
        self.prefix.deref()
    }
}
impl S3Location {
    /// Creates a new builder-style object to manufacture [`S3Location`](crate::types::S3Location).
    pub fn builder() -> crate::types::builders::S3LocationBuilder {
        crate::types::builders::S3LocationBuilder::default()
    }
}

/// A builder for [`S3Location`](crate::types::S3Location).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct S3LocationBuilder {
    pub(crate) bucket: ::std::option::Option<::std::string::String>,
    pub(crate) prefix: ::std::option::Option<::std::string::String>,
}
impl S3LocationBuilder {
    /// <p>The name of the Amazon S3 bucket. This bucket contains the stored data.</p>
    /// This field is required.
    pub fn bucket(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.bucket = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The name of the Amazon S3 bucket. This bucket contains the stored data.</p>
    pub fn set_bucket(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.bucket = input;
        self
    }
    /// <p>The name of the Amazon S3 bucket. This bucket contains the stored data.</p>
    pub fn get_bucket(&self) -> &::std::option::Option<::std::string::String> {
        &self.bucket
    }
    /// <p>The prefix for objects in the Amazon S3 bucket. This prefix is added to the object keys to organize the data.</p>
    /// This field is required.
    pub fn prefix(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.prefix = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The prefix for objects in the Amazon S3 bucket. This prefix is added to the object keys to organize the data.</p>
    pub fn set_prefix(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.prefix = input;
        self
    }
    /// <p>The prefix for objects in the Amazon S3 bucket. This prefix is added to the object keys to organize the data.</p>
    pub fn get_prefix(&self) -> &::std::option::Option<::std::string::String> {
        &self.prefix
    }
    /// Consumes the builder and constructs a [`S3Location`](crate::types::S3Location).
    /// This method will fail if any of the following fields are not set:
    /// - [`bucket`](crate::types::builders::S3LocationBuilder::bucket)
    /// - [`prefix`](crate::types::builders::S3LocationBuilder::prefix)
    pub fn build(self) -> ::std::result::Result<crate::types::S3Location, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::S3Location {
            bucket: self.bucket.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "bucket",
                    "bucket was not specified but it is required when building S3Location",
                )
            })?,
            prefix: self.prefix.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "prefix",
                    "prefix was not specified but it is required when building S3Location",
                )
            })?,
        })
    }
}
