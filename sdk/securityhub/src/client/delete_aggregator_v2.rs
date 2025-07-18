// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DeleteAggregatorV2`](crate::operation::delete_aggregator_v2::builders::DeleteAggregatorV2FluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`aggregator_v2_arn(impl Into<String>)`](crate::operation::delete_aggregator_v2::builders::DeleteAggregatorV2FluentBuilder::aggregator_v2_arn) / [`set_aggregator_v2_arn(Option<String>)`](crate::operation::delete_aggregator_v2::builders::DeleteAggregatorV2FluentBuilder::set_aggregator_v2_arn):<br>required: **true**<br><p>The ARN of the Aggregator V2.</p><br>
    /// - On success, responds with [`DeleteAggregatorV2Output`](crate::operation::delete_aggregator_v2::DeleteAggregatorV2Output)
    /// - On failure, responds with [`SdkError<DeleteAggregatorV2Error>`](crate::operation::delete_aggregator_v2::DeleteAggregatorV2Error)
    pub fn delete_aggregator_v2(&self) -> crate::operation::delete_aggregator_v2::builders::DeleteAggregatorV2FluentBuilder {
        crate::operation::delete_aggregator_v2::builders::DeleteAggregatorV2FluentBuilder::new(self.handle.clone())
    }
}
