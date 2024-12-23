// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListAudienceExportJobs`](crate::operation::list_audience_export_jobs::builders::ListAudienceExportJobsFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_audience_export_jobs::builders::ListAudienceExportJobsFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_audience_export_jobs::builders::ListAudienceExportJobsFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_audience_export_jobs::builders::ListAudienceExportJobsFluentBuilder::set_next_token):<br>required: **false**<br><p>The token value retrieved from a previous call to access the next page of results.</p><br>
    ///   - [`max_results(i32)`](crate::operation::list_audience_export_jobs::builders::ListAudienceExportJobsFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_audience_export_jobs::builders::ListAudienceExportJobsFluentBuilder::set_max_results):<br>required: **false**<br><p>The maximum size of the results that is returned per call.</p><br>
    ///   - [`audience_generation_job_arn(impl Into<String>)`](crate::operation::list_audience_export_jobs::builders::ListAudienceExportJobsFluentBuilder::audience_generation_job_arn) / [`set_audience_generation_job_arn(Option<String>)`](crate::operation::list_audience_export_jobs::builders::ListAudienceExportJobsFluentBuilder::set_audience_generation_job_arn):<br>required: **false**<br><p>The Amazon Resource Name (ARN) of the audience generation job that you are interested in.</p><br>
    /// - On success, responds with [`ListAudienceExportJobsOutput`](crate::operation::list_audience_export_jobs::ListAudienceExportJobsOutput) with field(s):
    ///   - [`next_token(Option<String>)`](crate::operation::list_audience_export_jobs::ListAudienceExportJobsOutput::next_token): <p>The token value used to access the next page of results.</p>
    ///   - [`audience_export_jobs(Vec::<AudienceExportJobSummary>)`](crate::operation::list_audience_export_jobs::ListAudienceExportJobsOutput::audience_export_jobs): <p>The audience export jobs that match the request.</p>
    /// - On failure, responds with [`SdkError<ListAudienceExportJobsError>`](crate::operation::list_audience_export_jobs::ListAudienceExportJobsError)
    pub fn list_audience_export_jobs(&self) -> crate::operation::list_audience_export_jobs::builders::ListAudienceExportJobsFluentBuilder {
        crate::operation::list_audience_export_jobs::builders::ListAudienceExportJobsFluentBuilder::new(self.handle.clone())
    }
}
