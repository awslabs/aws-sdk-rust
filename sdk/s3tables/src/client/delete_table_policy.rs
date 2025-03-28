// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DeleteTablePolicy`](crate::operation::delete_table_policy::builders::DeleteTablePolicyFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`table_bucket_arn(impl Into<String>)`](crate::operation::delete_table_policy::builders::DeleteTablePolicyFluentBuilder::table_bucket_arn) / [`set_table_bucket_arn(Option<String>)`](crate::operation::delete_table_policy::builders::DeleteTablePolicyFluentBuilder::set_table_bucket_arn):<br>required: **true**<br><p>The Amazon Resource Name (ARN) of the table bucket that contains the table.</p><br>
    ///   - [`namespace(impl Into<String>)`](crate::operation::delete_table_policy::builders::DeleteTablePolicyFluentBuilder::namespace) / [`set_namespace(Option<String>)`](crate::operation::delete_table_policy::builders::DeleteTablePolicyFluentBuilder::set_namespace):<br>required: **true**<br><p>The namespace associated with the table.</p><br>
    ///   - [`name(impl Into<String>)`](crate::operation::delete_table_policy::builders::DeleteTablePolicyFluentBuilder::name) / [`set_name(Option<String>)`](crate::operation::delete_table_policy::builders::DeleteTablePolicyFluentBuilder::set_name):<br>required: **true**<br><p>The table name.</p><br>
    /// - On success, responds with [`DeleteTablePolicyOutput`](crate::operation::delete_table_policy::DeleteTablePolicyOutput)
    /// - On failure, responds with [`SdkError<DeleteTablePolicyError>`](crate::operation::delete_table_policy::DeleteTablePolicyError)
    pub fn delete_table_policy(&self) -> crate::operation::delete_table_policy::builders::DeleteTablePolicyFluentBuilder {
        crate::operation::delete_table_policy::builders::DeleteTablePolicyFluentBuilder::new(self.handle.clone())
    }
}
