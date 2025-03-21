// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DescribeVpcBlockPublicAccessOptions`](crate::operation::describe_vpc_block_public_access_options::builders::DescribeVpcBlockPublicAccessOptionsFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`dry_run(bool)`](crate::operation::describe_vpc_block_public_access_options::builders::DescribeVpcBlockPublicAccessOptionsFluentBuilder::dry_run) / [`set_dry_run(Option<bool>)`](crate::operation::describe_vpc_block_public_access_options::builders::DescribeVpcBlockPublicAccessOptionsFluentBuilder::set_dry_run):<br>required: **false**<br><p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p><br>
    /// - On success, responds with [`DescribeVpcBlockPublicAccessOptionsOutput`](crate::operation::describe_vpc_block_public_access_options::DescribeVpcBlockPublicAccessOptionsOutput) with field(s):
    ///   - [`vpc_block_public_access_options(Option<VpcBlockPublicAccessOptions>)`](crate::operation::describe_vpc_block_public_access_options::DescribeVpcBlockPublicAccessOptionsOutput::vpc_block_public_access_options): <p>Details related to the options.</p>
    /// - On failure, responds with [`SdkError<DescribeVpcBlockPublicAccessOptionsError>`](crate::operation::describe_vpc_block_public_access_options::DescribeVpcBlockPublicAccessOptionsError)
    pub fn describe_vpc_block_public_access_options(
        &self,
    ) -> crate::operation::describe_vpc_block_public_access_options::builders::DescribeVpcBlockPublicAccessOptionsFluentBuilder {
        crate::operation::describe_vpc_block_public_access_options::builders::DescribeVpcBlockPublicAccessOptionsFluentBuilder::new(
            self.handle.clone(),
        )
    }
}
