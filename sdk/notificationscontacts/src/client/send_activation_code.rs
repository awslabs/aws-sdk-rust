// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`SendActivationCode`](crate::operation::send_activation_code::builders::SendActivationCodeFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`arn(impl Into<String>)`](crate::operation::send_activation_code::builders::SendActivationCodeFluentBuilder::arn) / [`set_arn(Option<String>)`](crate::operation::send_activation_code::builders::SendActivationCodeFluentBuilder::set_arn):<br>required: **true**<br><p>The Amazon Resource Name (ARN) of the resource.</p><br>
    /// - On success, responds with [`SendActivationCodeOutput`](crate::operation::send_activation_code::SendActivationCodeOutput)
    /// - On failure, responds with [`SdkError<SendActivationCodeError>`](crate::operation::send_activation_code::SendActivationCodeError)
    pub fn send_activation_code(&self) -> crate::operation::send_activation_code::builders::SendActivationCodeFluentBuilder {
        crate::operation::send_activation_code::builders::SendActivationCodeFluentBuilder::new(self.handle.clone())
    }
}
