// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetAIAgent`](crate::operation::get_ai_agent::builders::GetAIAgentFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`assistant_id(impl Into<String>)`](crate::operation::get_ai_agent::builders::GetAIAgentFluentBuilder::assistant_id) / [`set_assistant_id(Option<String>)`](crate::operation::get_ai_agent::builders::GetAIAgentFluentBuilder::set_assistant_id):<br>required: **true**<br><p>The identifier of the Amazon Q in Connect assistant. Can be either the ID or the ARN. URLs cannot contain the ARN.</p><br>
    ///   - [`ai_agent_id(impl Into<String>)`](crate::operation::get_ai_agent::builders::GetAIAgentFluentBuilder::ai_agent_id) / [`set_ai_agent_id(Option<String>)`](crate::operation::get_ai_agent::builders::GetAIAgentFluentBuilder::set_ai_agent_id):<br>required: **true**<br><p>The identifier of the Amazon Q in Connect AI Agent (with or without a version qualifier). Can be either the ID or the ARN. URLs cannot contain the ARN.</p><br>
    /// - On success, responds with [`GetAiAgentOutput`](crate::operation::get_ai_agent::GetAiAgentOutput) with field(s):
    ///   - [`ai_agent(Option<AiAgentData>)`](crate::operation::get_ai_agent::GetAiAgentOutput::ai_agent): <p>The data of the AI Agent.</p>
    ///   - [`version_number(Option<i64>)`](crate::operation::get_ai_agent::GetAiAgentOutput::version_number): <p>The version number of the AI Agent version (returned if an AI Agent version was specified via use of a qualifier for the <code>aiAgentId</code> on the request).</p>
    /// - On failure, responds with [`SdkError<GetAIAgentError>`](crate::operation::get_ai_agent::GetAIAgentError)
    pub fn get_ai_agent(&self) -> crate::operation::get_ai_agent::builders::GetAIAgentFluentBuilder {
        crate::operation::get_ai_agent::builders::GetAIAgentFluentBuilder::new(self.handle.clone())
    }
}
