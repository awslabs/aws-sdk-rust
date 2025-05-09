// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetAuthPolicy`](crate::operation::get_auth_policy::builders::GetAuthPolicyFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`resource_identifier(impl Into<String>)`](crate::operation::get_auth_policy::builders::GetAuthPolicyFluentBuilder::resource_identifier) / [`set_resource_identifier(Option<String>)`](crate::operation::get_auth_policy::builders::GetAuthPolicyFluentBuilder::set_resource_identifier):<br>required: **true**<br><p>The ID or ARN of the service network or service.</p><br>
    /// - On success, responds with [`GetAuthPolicyOutput`](crate::operation::get_auth_policy::GetAuthPolicyOutput) with field(s):
    ///   - [`policy(Option<String>)`](crate::operation::get_auth_policy::GetAuthPolicyOutput::policy): <p>The auth policy.</p>
    ///   - [`state(Option<AuthPolicyState>)`](crate::operation::get_auth_policy::GetAuthPolicyOutput::state): <p>The state of the auth policy. The auth policy is only active when the auth type is set to <code>AWS_IAM</code>. If you provide a policy, then authentication and authorization decisions are made based on this policy and the client's IAM policy. If the auth type is <code>NONE</code>, then any auth policy that you provide remains inactive. For more information, see <a href="https://docs.aws.amazon.com/vpc-lattice/latest/ug/service-networks.html#create-service-network">Create a service network</a> in the <i>Amazon VPC Lattice User Guide</i>.</p>
    ///   - [`created_at(Option<DateTime>)`](crate::operation::get_auth_policy::GetAuthPolicyOutput::created_at): <p>The date and time that the auth policy was created, in ISO-8601 format.</p>
    ///   - [`last_updated_at(Option<DateTime>)`](crate::operation::get_auth_policy::GetAuthPolicyOutput::last_updated_at): <p>The date and time that the auth policy was last updated, in ISO-8601 format.</p>
    /// - On failure, responds with [`SdkError<GetAuthPolicyError>`](crate::operation::get_auth_policy::GetAuthPolicyError)
    pub fn get_auth_policy(&self) -> crate::operation::get_auth_policy::builders::GetAuthPolicyFluentBuilder {
        crate::operation::get_auth_policy::builders::GetAuthPolicyFluentBuilder::new(self.handle.clone())
    }
}
