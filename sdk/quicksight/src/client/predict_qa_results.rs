// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`PredictQAResults`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`aws_account_id(impl Into<String>)`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::aws_account_id) / [`set_aws_account_id(Option<String>)`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::set_aws_account_id):<br>required: **true**<br><p>The ID of the Amazon Web Services account that the user wants to execute Predict QA results in.</p><br>
    ///   - [`query_text(impl Into<String>)`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::query_text) / [`set_query_text(Option<String>)`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::set_query_text):<br>required: **true**<br><p>The query text to be used to predict QA results.</p><br>
    ///   - [`include_quick_sight_q_index(IncludeQuickSightQIndex)`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::include_quick_sight_q_index) / [`set_include_quick_sight_q_index(Option<IncludeQuickSightQIndex>)`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::set_include_quick_sight_q_index):<br>required: **false**<br><p>Indicates whether Q indicies are included or excluded.</p><br>
    ///   - [`include_generated_answer(IncludeGeneratedAnswer)`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::include_generated_answer) / [`set_include_generated_answer(Option<IncludeGeneratedAnswer>)`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::set_include_generated_answer):<br>required: **false**<br><p>Indicates whether generated answers are included or excluded.</p><br>
    ///   - [`max_topics_to_consider(i32)`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::max_topics_to_consider) / [`set_max_topics_to_consider(Option<i32>)`](crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::set_max_topics_to_consider):<br>required: **false**<br><p>The number of maximum topics to be considered to predict QA results.</p><br>
    /// - On success, responds with [`PredictQaResultsOutput`](crate::operation::predict_qa_results::PredictQaResultsOutput) with field(s):
    ///   - [`primary_result(Option<QaResult>)`](crate::operation::predict_qa_results::PredictQaResultsOutput::primary_result): <p>The primary visual response.</p>
    ///   - [`additional_results(Option<Vec::<QaResult>>)`](crate::operation::predict_qa_results::PredictQaResultsOutput::additional_results): <p>Additional visual responses.</p>
    ///   - [`request_id(Option<String>)`](crate::operation::predict_qa_results::PredictQaResultsOutput::request_id): <p>The Amazon Web Services request ID for this operation.</p>
    ///   - [`status(i32)`](crate::operation::predict_qa_results::PredictQaResultsOutput::status): <p>The HTTP status of the request.</p>
    /// - On failure, responds with [`SdkError<PredictQAResultsError>`](crate::operation::predict_qa_results::PredictQAResultsError)
    pub fn predict_qa_results(&self) -> crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder {
        crate::operation::predict_qa_results::builders::PredictQAResultsFluentBuilder::new(self.handle.clone())
    }
}
