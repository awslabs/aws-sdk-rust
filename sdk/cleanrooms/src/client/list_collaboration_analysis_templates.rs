// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListCollaborationAnalysisTemplates`](crate::operation::list_collaboration_analysis_templates::builders::ListCollaborationAnalysisTemplatesFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_collaboration_analysis_templates::builders::ListCollaborationAnalysisTemplatesFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`collaboration_identifier(impl Into<String>)`](crate::operation::list_collaboration_analysis_templates::builders::ListCollaborationAnalysisTemplatesFluentBuilder::collaboration_identifier) / [`set_collaboration_identifier(Option<String>)`](crate::operation::list_collaboration_analysis_templates::builders::ListCollaborationAnalysisTemplatesFluentBuilder::set_collaboration_identifier):<br>required: **true**<br><p>A unique identifier for the collaboration that the analysis templates belong to. Currently accepts collaboration ID.</p><br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_collaboration_analysis_templates::builders::ListCollaborationAnalysisTemplatesFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_collaboration_analysis_templates::builders::ListCollaborationAnalysisTemplatesFluentBuilder::set_next_token):<br>required: **false**<br><p>The pagination token that's used to fetch the next set of results.</p><br>
    ///   - [`max_results(i32)`](crate::operation::list_collaboration_analysis_templates::builders::ListCollaborationAnalysisTemplatesFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_collaboration_analysis_templates::builders::ListCollaborationAnalysisTemplatesFluentBuilder::set_max_results):<br>required: **false**<br><p>The maximum number of results that are returned for an API request call. The service chooses a default number if you don't set one. The service might return a `nextToken` even if the `maxResults` value has not been met.</p><br>
    /// - On success, responds with [`ListCollaborationAnalysisTemplatesOutput`](crate::operation::list_collaboration_analysis_templates::ListCollaborationAnalysisTemplatesOutput) with field(s):
    ///   - [`next_token(Option<String>)`](crate::operation::list_collaboration_analysis_templates::ListCollaborationAnalysisTemplatesOutput::next_token): <p>The pagination token that's used to fetch the next set of results.</p>
    ///   - [`collaboration_analysis_template_summaries(Vec::<CollaborationAnalysisTemplateSummary>)`](crate::operation::list_collaboration_analysis_templates::ListCollaborationAnalysisTemplatesOutput::collaboration_analysis_template_summaries): <p>The metadata of the analysis template within a collaboration.</p>
    /// - On failure, responds with [`SdkError<ListCollaborationAnalysisTemplatesError>`](crate::operation::list_collaboration_analysis_templates::ListCollaborationAnalysisTemplatesError)
    pub fn list_collaboration_analysis_templates(
        &self,
    ) -> crate::operation::list_collaboration_analysis_templates::builders::ListCollaborationAnalysisTemplatesFluentBuilder {
        crate::operation::list_collaboration_analysis_templates::builders::ListCollaborationAnalysisTemplatesFluentBuilder::new(self.handle.clone())
    }
}
