// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct DescribeCapacityBlocksOutput {
    /// <p>The Capacity Blocks.</p>
    pub capacity_blocks: ::std::option::Option<::std::vec::Vec<crate::types::CapacityBlock>>,
    /// <p>The token to use to retrieve the next page of results. This value is <code>null</code> when there are no more results to return.</p>
    pub next_token: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl DescribeCapacityBlocksOutput {
    /// <p>The Capacity Blocks.</p>
    ///
    /// If no value was sent for this field, a default will be set. If you want to determine if no value was sent, use `.capacity_blocks.is_none()`.
    pub fn capacity_blocks(&self) -> &[crate::types::CapacityBlock] {
        self.capacity_blocks.as_deref().unwrap_or_default()
    }
    /// <p>The token to use to retrieve the next page of results. This value is <code>null</code> when there are no more results to return.</p>
    pub fn next_token(&self) -> ::std::option::Option<&str> {
        self.next_token.as_deref()
    }
}
impl ::aws_types::request_id::RequestId for DescribeCapacityBlocksOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl DescribeCapacityBlocksOutput {
    /// Creates a new builder-style object to manufacture [`DescribeCapacityBlocksOutput`](crate::operation::describe_capacity_blocks::DescribeCapacityBlocksOutput).
    pub fn builder() -> crate::operation::describe_capacity_blocks::builders::DescribeCapacityBlocksOutputBuilder {
        crate::operation::describe_capacity_blocks::builders::DescribeCapacityBlocksOutputBuilder::default()
    }
}

/// A builder for [`DescribeCapacityBlocksOutput`](crate::operation::describe_capacity_blocks::DescribeCapacityBlocksOutput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct DescribeCapacityBlocksOutputBuilder {
    pub(crate) capacity_blocks: ::std::option::Option<::std::vec::Vec<crate::types::CapacityBlock>>,
    pub(crate) next_token: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl DescribeCapacityBlocksOutputBuilder {
    /// Appends an item to `capacity_blocks`.
    ///
    /// To override the contents of this collection use [`set_capacity_blocks`](Self::set_capacity_blocks).
    ///
    /// <p>The Capacity Blocks.</p>
    pub fn capacity_blocks(mut self, input: crate::types::CapacityBlock) -> Self {
        let mut v = self.capacity_blocks.unwrap_or_default();
        v.push(input);
        self.capacity_blocks = ::std::option::Option::Some(v);
        self
    }
    /// <p>The Capacity Blocks.</p>
    pub fn set_capacity_blocks(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::CapacityBlock>>) -> Self {
        self.capacity_blocks = input;
        self
    }
    /// <p>The Capacity Blocks.</p>
    pub fn get_capacity_blocks(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::CapacityBlock>> {
        &self.capacity_blocks
    }
    /// <p>The token to use to retrieve the next page of results. This value is <code>null</code> when there are no more results to return.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.next_token = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The token to use to retrieve the next page of results. This value is <code>null</code> when there are no more results to return.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.next_token = input;
        self
    }
    /// <p>The token to use to retrieve the next page of results. This value is <code>null</code> when there are no more results to return.</p>
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
    /// Consumes the builder and constructs a [`DescribeCapacityBlocksOutput`](crate::operation::describe_capacity_blocks::DescribeCapacityBlocksOutput).
    pub fn build(self) -> crate::operation::describe_capacity_blocks::DescribeCapacityBlocksOutput {
        crate::operation::describe_capacity_blocks::DescribeCapacityBlocksOutput {
            capacity_blocks: self.capacity_blocks,
            next_token: self.next_token,
            _request_id: self._request_id,
        }
    }
}
