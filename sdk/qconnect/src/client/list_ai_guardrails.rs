// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListAIGuardrails`](crate::operation::list_ai_guardrails::builders::ListAIGuardrailsFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_ai_guardrails::builders::ListAIGuardrailsFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`assistant_id(impl Into<String>)`](crate::operation::list_ai_guardrails::builders::ListAIGuardrailsFluentBuilder::assistant_id) / [`set_assistant_id(Option<String>)`](crate::operation::list_ai_guardrails::builders::ListAIGuardrailsFluentBuilder::set_assistant_id):<br>required: **true**<br><p>The identifier of the Amazon Q in Connect assistant. Can be either the ID or the ARN. URLs cannot contain the ARN.</p><br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_ai_guardrails::builders::ListAIGuardrailsFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_ai_guardrails::builders::ListAIGuardrailsFluentBuilder::set_next_token):<br>required: **false**<br><p>The token for the next set of results. Use the value returned in the previous response in the next request to retrieve the next set of results.</p><br>
    ///   - [`max_results(i32)`](crate::operation::list_ai_guardrails::builders::ListAIGuardrailsFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_ai_guardrails::builders::ListAIGuardrailsFluentBuilder::set_max_results):<br>required: **false**<br><p>The maximum number of results to return per page.</p><br>
    /// - On success, responds with [`ListAiGuardrailsOutput`](crate::operation::list_ai_guardrails::ListAiGuardrailsOutput) with field(s):
    ///   - [`ai_guardrail_summaries(Vec::<AiGuardrailSummary>)`](crate::operation::list_ai_guardrails::ListAiGuardrailsOutput::ai_guardrail_summaries): <p>The summaries of the AI Guardrails.</p>
    ///   - [`next_token(Option<String>)`](crate::operation::list_ai_guardrails::ListAiGuardrailsOutput::next_token): <p>The token for the next set of results. Use the value returned in the previous response in the next request to retrieve the next set of results.</p>
    /// - On failure, responds with [`SdkError<ListAIGuardrailsError>`](crate::operation::list_ai_guardrails::ListAIGuardrailsError)
    pub fn list_ai_guardrails(&self) -> crate::operation::list_ai_guardrails::builders::ListAIGuardrailsFluentBuilder {
        crate::operation::list_ai_guardrails::builders::ListAIGuardrailsFluentBuilder::new(self.handle.clone())
    }
}
