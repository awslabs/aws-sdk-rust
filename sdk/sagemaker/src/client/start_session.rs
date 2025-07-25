// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`StartSession`](crate::operation::start_session::builders::StartSessionFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`resource_identifier(impl Into<String>)`](crate::operation::start_session::builders::StartSessionFluentBuilder::resource_identifier) / [`set_resource_identifier(Option<String>)`](crate::operation::start_session::builders::StartSessionFluentBuilder::set_resource_identifier):<br>required: **true**<br><p>The Amazon Resource Name (ARN) of the resource to which the remote connection will be established. For example, this identifies the specific ARN space application you want to connect to from your local IDE.</p><br>
    /// - On success, responds with [`StartSessionOutput`](crate::operation::start_session::StartSessionOutput) with field(s):
    ///   - [`session_id(Option<String>)`](crate::operation::start_session::StartSessionOutput::session_id): <p>A unique identifier for the established remote connection session.</p>
    ///   - [`stream_url(Option<String>)`](crate::operation::start_session::StartSessionOutput::stream_url): <p>A WebSocket URL used to establish a SSH connection between the local IDE and remote SageMaker space.</p>
    ///   - [`token_value(Option<String>)`](crate::operation::start_session::StartSessionOutput::token_value): <p>An encrypted token value containing session and caller information.</p>
    /// - On failure, responds with [`SdkError<StartSessionError>`](crate::operation::start_session::StartSessionError)
    pub fn start_session(&self) -> crate::operation::start_session::builders::StartSessionFluentBuilder {
        crate::operation::start_session::builders::StartSessionFluentBuilder::new(self.handle.clone())
    }
}
