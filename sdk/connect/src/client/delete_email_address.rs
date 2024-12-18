// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DeleteEmailAddress`](crate::operation::delete_email_address::builders::DeleteEmailAddressFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`instance_id(impl Into<String>)`](crate::operation::delete_email_address::builders::DeleteEmailAddressFluentBuilder::instance_id) / [`set_instance_id(Option<String>)`](crate::operation::delete_email_address::builders::DeleteEmailAddressFluentBuilder::set_instance_id):<br>required: **true**<br><p>The identifier of the Amazon Connect instance. You can <a href="https://docs.aws.amazon.com/connect/latest/adminguide/find-instance-arn.html">find the instance ID</a> in the Amazon Resource Name (ARN) of the instance.</p><br>
    ///   - [`email_address_id(impl Into<String>)`](crate::operation::delete_email_address::builders::DeleteEmailAddressFluentBuilder::email_address_id) / [`set_email_address_id(Option<String>)`](crate::operation::delete_email_address::builders::DeleteEmailAddressFluentBuilder::set_email_address_id):<br>required: **true**<br><p>The identifier of the email address.</p><br>
    /// - On success, responds with [`DeleteEmailAddressOutput`](crate::operation::delete_email_address::DeleteEmailAddressOutput)
    /// - On failure, responds with [`SdkError<DeleteEmailAddressError>`](crate::operation::delete_email_address::DeleteEmailAddressError)
    pub fn delete_email_address(&self) -> crate::operation::delete_email_address::builders::DeleteEmailAddressFluentBuilder {
        crate::operation::delete_email_address::builders::DeleteEmailAddressFluentBuilder::new(self.handle.clone())
    }
}
