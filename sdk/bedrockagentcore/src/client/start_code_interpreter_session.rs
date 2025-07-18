// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`StartCodeInterpreterSession`](crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`code_interpreter_identifier(impl Into<String>)`](crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder::code_interpreter_identifier) / [`set_code_interpreter_identifier(Option<String>)`](crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder::set_code_interpreter_identifier):<br>required: **true**<br><p>The unique identifier of the code interpreter to use for this session. This identifier specifies which code interpreter environment to initialize for the session.</p><br>
    ///   - [`name(impl Into<String>)`](crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder::name) / [`set_name(Option<String>)`](crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder::set_name):<br>required: **false**<br><p>The name of the code interpreter session. This name helps you identify and manage the session. The name does not need to be unique.</p><br>
    ///   - [`session_timeout_seconds(i32)`](crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder::session_timeout_seconds) / [`set_session_timeout_seconds(Option<i32>)`](crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder::set_session_timeout_seconds):<br>required: **false**<br><p>The time in seconds after which the session automatically terminates if there is no activity. The default value is 3600 seconds (1 hour). The minimum allowed value is 60 seconds, and the maximum allowed value is 28800 seconds (8 hours).</p><br>
    ///   - [`client_token(impl Into<String>)`](crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder::client_token) / [`set_client_token(Option<String>)`](crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder::set_client_token):<br>required: **false**<br><p>A unique, case-sensitive identifier to ensure that the API request completes no more than one time. If this token matches a previous request, Amazon Bedrock ignores the request, but does not return an error. This parameter helps prevent the creation of duplicate sessions if there are temporary network issues.</p><br>
    /// - On success, responds with [`StartCodeInterpreterSessionOutput`](crate::operation::start_code_interpreter_session::StartCodeInterpreterSessionOutput) with field(s):
    ///   - [`code_interpreter_identifier(String)`](crate::operation::start_code_interpreter_session::StartCodeInterpreterSessionOutput::code_interpreter_identifier): <p>The identifier of the code interpreter.</p>
    ///   - [`session_id(String)`](crate::operation::start_code_interpreter_session::StartCodeInterpreterSessionOutput::session_id): <p>The unique identifier of the created code interpreter session.</p>
    ///   - [`created_at(DateTime)`](crate::operation::start_code_interpreter_session::StartCodeInterpreterSessionOutput::created_at): <p>The time at which the code interpreter session was created.</p>
    /// - On failure, responds with [`SdkError<StartCodeInterpreterSessionError>`](crate::operation::start_code_interpreter_session::StartCodeInterpreterSessionError)
    pub fn start_code_interpreter_session(
        &self,
    ) -> crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder {
        crate::operation::start_code_interpreter_session::builders::StartCodeInterpreterSessionFluentBuilder::new(self.handle.clone())
    }
}
