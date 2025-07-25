// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListWhatsAppMessageTemplates`](crate::operation::list_whats_app_message_templates::builders::ListWhatsAppMessageTemplatesFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_whats_app_message_templates::builders::ListWhatsAppMessageTemplatesFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`id(impl Into<String>)`](crate::operation::list_whats_app_message_templates::builders::ListWhatsAppMessageTemplatesFluentBuilder::id) / [`set_id(Option<String>)`](crate::operation::list_whats_app_message_templates::builders::ListWhatsAppMessageTemplatesFluentBuilder::set_id):<br>required: **true**<br><p>The ID of the WhatsApp Business Account to list templates for.</p><br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_whats_app_message_templates::builders::ListWhatsAppMessageTemplatesFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_whats_app_message_templates::builders::ListWhatsAppMessageTemplatesFluentBuilder::set_next_token):<br>required: **false**<br><p>The token for the next page of results.</p><br>
    ///   - [`max_results(i32)`](crate::operation::list_whats_app_message_templates::builders::ListWhatsAppMessageTemplatesFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_whats_app_message_templates::builders::ListWhatsAppMessageTemplatesFluentBuilder::set_max_results):<br>required: **false**<br><p>The maximum number of results to return per page (1-100).</p><br>
    /// - On success, responds with [`ListWhatsAppMessageTemplatesOutput`](crate::operation::list_whats_app_message_templates::ListWhatsAppMessageTemplatesOutput) with field(s):
    ///   - [`templates(Option<Vec::<TemplateSummary>>)`](crate::operation::list_whats_app_message_templates::ListWhatsAppMessageTemplatesOutput::templates): <p>A list of template summaries.</p>
    ///   - [`next_token(Option<String>)`](crate::operation::list_whats_app_message_templates::ListWhatsAppMessageTemplatesOutput::next_token): <p>The token to retrieve the next page of results, if any.</p>
    /// - On failure, responds with [`SdkError<ListWhatsAppMessageTemplatesError>`](crate::operation::list_whats_app_message_templates::ListWhatsAppMessageTemplatesError)
    pub fn list_whats_app_message_templates(
        &self,
    ) -> crate::operation::list_whats_app_message_templates::builders::ListWhatsAppMessageTemplatesFluentBuilder {
        crate::operation::list_whats_app_message_templates::builders::ListWhatsAppMessageTemplatesFluentBuilder::new(self.handle.clone())
    }
}
