// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DeleteAIPromptVersion`](crate::operation::delete_ai_prompt_version::builders::DeleteAIPromptVersionFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`assistant_id(impl Into<String>)`](crate::operation::delete_ai_prompt_version::builders::DeleteAIPromptVersionFluentBuilder::assistant_id) / [`set_assistant_id(Option<String>)`](crate::operation::delete_ai_prompt_version::builders::DeleteAIPromptVersionFluentBuilder::set_assistant_id):<br>required: **true**<br><p>The identifier of the Amazon Q in Connect assistant. Can be either the ID or the ARN. URLs cannot contain the ARN.</p><br>
    ///   - [`ai_prompt_id(impl Into<String>)`](crate::operation::delete_ai_prompt_version::builders::DeleteAIPromptVersionFluentBuilder::ai_prompt_id) / [`set_ai_prompt_id(Option<String>)`](crate::operation::delete_ai_prompt_version::builders::DeleteAIPromptVersionFluentBuilder::set_ai_prompt_id):<br>required: **true**<br><p>The identifier of the Amazon Q in Connect AI prompt.</p><br>
    ///   - [`version_number(i64)`](crate::operation::delete_ai_prompt_version::builders::DeleteAIPromptVersionFluentBuilder::version_number) / [`set_version_number(Option<i64>)`](crate::operation::delete_ai_prompt_version::builders::DeleteAIPromptVersionFluentBuilder::set_version_number):<br>required: **true**<br><p>The version number of the AI Prompt version to be deleted.</p><br>
    /// - On success, responds with [`DeleteAiPromptVersionOutput`](crate::operation::delete_ai_prompt_version::DeleteAiPromptVersionOutput)
    /// - On failure, responds with [`SdkError<DeleteAIPromptVersionError>`](crate::operation::delete_ai_prompt_version::DeleteAIPromptVersionError)
    pub fn delete_ai_prompt_version(&self) -> crate::operation::delete_ai_prompt_version::builders::DeleteAIPromptVersionFluentBuilder {
        crate::operation::delete_ai_prompt_version::builders::DeleteAIPromptVersionFluentBuilder::new(self.handle.clone())
    }
}
