// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DeleteConfiguredModelAlgorithm`](crate::operation::delete_configured_model_algorithm::builders::DeleteConfiguredModelAlgorithmFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`configured_model_algorithm_arn(impl Into<String>)`](crate::operation::delete_configured_model_algorithm::builders::DeleteConfiguredModelAlgorithmFluentBuilder::configured_model_algorithm_arn) / [`set_configured_model_algorithm_arn(Option<String>)`](crate::operation::delete_configured_model_algorithm::builders::DeleteConfiguredModelAlgorithmFluentBuilder::set_configured_model_algorithm_arn):<br>required: **true**<br><p>The Amazon Resource Name (ARN) of the configured model algorithm that you want to delete.</p><br>
    /// - On success, responds with [`DeleteConfiguredModelAlgorithmOutput`](crate::operation::delete_configured_model_algorithm::DeleteConfiguredModelAlgorithmOutput)
    /// - On failure, responds with [`SdkError<DeleteConfiguredModelAlgorithmError>`](crate::operation::delete_configured_model_algorithm::DeleteConfiguredModelAlgorithmError)
    pub fn delete_configured_model_algorithm(
        &self,
    ) -> crate::operation::delete_configured_model_algorithm::builders::DeleteConfiguredModelAlgorithmFluentBuilder {
        crate::operation::delete_configured_model_algorithm::builders::DeleteConfiguredModelAlgorithmFluentBuilder::new(self.handle.clone())
    }
}
