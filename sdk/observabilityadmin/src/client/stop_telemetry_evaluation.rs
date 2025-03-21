// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`StopTelemetryEvaluation`](crate::operation::stop_telemetry_evaluation::builders::StopTelemetryEvaluationFluentBuilder) operation.
    ///
    /// - The fluent builder takes no input, just [`send`](crate::operation::stop_telemetry_evaluation::builders::StopTelemetryEvaluationFluentBuilder::send) it.
    /// - On success, responds with [`StopTelemetryEvaluationOutput`](crate::operation::stop_telemetry_evaluation::StopTelemetryEvaluationOutput)
    /// - On failure, responds with [`SdkError<StopTelemetryEvaluationError>`](crate::operation::stop_telemetry_evaluation::StopTelemetryEvaluationError)
    pub fn stop_telemetry_evaluation(&self) -> crate::operation::stop_telemetry_evaluation::builders::StopTelemetryEvaluationFluentBuilder {
        crate::operation::stop_telemetry_evaluation::builders::StopTelemetryEvaluationFluentBuilder::new(self.handle.clone())
    }
}
