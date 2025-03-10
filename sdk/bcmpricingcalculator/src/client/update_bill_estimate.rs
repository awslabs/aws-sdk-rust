// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`UpdateBillEstimate`](crate::operation::update_bill_estimate::builders::UpdateBillEstimateFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`identifier(impl Into<String>)`](crate::operation::update_bill_estimate::builders::UpdateBillEstimateFluentBuilder::identifier) / [`set_identifier(Option<String>)`](crate::operation::update_bill_estimate::builders::UpdateBillEstimateFluentBuilder::set_identifier):<br>required: **true**<br><p>The unique identifier of the bill estimate to update.</p><br>
    ///   - [`name(impl Into<String>)`](crate::operation::update_bill_estimate::builders::UpdateBillEstimateFluentBuilder::name) / [`set_name(Option<String>)`](crate::operation::update_bill_estimate::builders::UpdateBillEstimateFluentBuilder::set_name):<br>required: **false**<br><p>The new name for the bill estimate.</p><br>
    ///   - [`expires_at(DateTime)`](crate::operation::update_bill_estimate::builders::UpdateBillEstimateFluentBuilder::expires_at) / [`set_expires_at(Option<DateTime>)`](crate::operation::update_bill_estimate::builders::UpdateBillEstimateFluentBuilder::set_expires_at):<br>required: **false**<br><p>The new expiration date for the bill estimate.</p><br>
    /// - On success, responds with [`UpdateBillEstimateOutput`](crate::operation::update_bill_estimate::UpdateBillEstimateOutput) with field(s):
    ///   - [`id(String)`](crate::operation::update_bill_estimate::UpdateBillEstimateOutput::id): <p>The unique identifier of the updated bill estimate.</p>
    ///   - [`name(Option<String>)`](crate::operation::update_bill_estimate::UpdateBillEstimateOutput::name): <p>The updated name of the bill estimate.</p>
    ///   - [`status(Option<BillEstimateStatus>)`](crate::operation::update_bill_estimate::UpdateBillEstimateOutput::status): <p>The current status of the updated bill estimate.</p>
    ///   - [`failure_message(Option<String>)`](crate::operation::update_bill_estimate::UpdateBillEstimateOutput::failure_message): <p>An error message if the bill estimate update failed.</p>
    ///   - [`bill_interval(Option<BillInterval>)`](crate::operation::update_bill_estimate::UpdateBillEstimateOutput::bill_interval): <p>The time period covered by the updated bill estimate.</p>
    ///   - [`cost_summary(Option<BillEstimateCostSummary>)`](crate::operation::update_bill_estimate::UpdateBillEstimateOutput::cost_summary): <p>A summary of the updated estimated costs.</p>
    ///   - [`created_at(Option<DateTime>)`](crate::operation::update_bill_estimate::UpdateBillEstimateOutput::created_at): <p>The timestamp when the bill estimate was originally created.</p>
    ///   - [`expires_at(Option<DateTime>)`](crate::operation::update_bill_estimate::UpdateBillEstimateOutput::expires_at): <p>The updated expiration timestamp for the bill estimate.</p>
    /// - On failure, responds with [`SdkError<UpdateBillEstimateError>`](crate::operation::update_bill_estimate::UpdateBillEstimateError)
    pub fn update_bill_estimate(&self) -> crate::operation::update_bill_estimate::builders::UpdateBillEstimateFluentBuilder {
        crate::operation::update_bill_estimate::builders::UpdateBillEstimateFluentBuilder::new(self.handle.clone())
    }
}
