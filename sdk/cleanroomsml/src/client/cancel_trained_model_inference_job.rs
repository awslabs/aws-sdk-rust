// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`CancelTrainedModelInferenceJob`](crate::operation::cancel_trained_model_inference_job::builders::CancelTrainedModelInferenceJobFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`membership_identifier(impl Into<String>)`](crate::operation::cancel_trained_model_inference_job::builders::CancelTrainedModelInferenceJobFluentBuilder::membership_identifier) / [`set_membership_identifier(Option<String>)`](crate::operation::cancel_trained_model_inference_job::builders::CancelTrainedModelInferenceJobFluentBuilder::set_membership_identifier):<br>required: **true**<br><p>The membership ID of the trained model inference job that you want to cancel.</p><br>
    ///   - [`trained_model_inference_job_arn(impl Into<String>)`](crate::operation::cancel_trained_model_inference_job::builders::CancelTrainedModelInferenceJobFluentBuilder::trained_model_inference_job_arn) / [`set_trained_model_inference_job_arn(Option<String>)`](crate::operation::cancel_trained_model_inference_job::builders::CancelTrainedModelInferenceJobFluentBuilder::set_trained_model_inference_job_arn):<br>required: **true**<br><p>The Amazon Resource Name (ARN) of the trained model inference job that you want to cancel.</p><br>
    /// - On success, responds with [`CancelTrainedModelInferenceJobOutput`](crate::operation::cancel_trained_model_inference_job::CancelTrainedModelInferenceJobOutput)
    /// - On failure, responds with [`SdkError<CancelTrainedModelInferenceJobError>`](crate::operation::cancel_trained_model_inference_job::CancelTrainedModelInferenceJobError)
    pub fn cancel_trained_model_inference_job(
        &self,
    ) -> crate::operation::cancel_trained_model_inference_job::builders::CancelTrainedModelInferenceJobFluentBuilder {
        crate::operation::cancel_trained_model_inference_job::builders::CancelTrainedModelInferenceJobFluentBuilder::new(self.handle.clone())
    }
}
