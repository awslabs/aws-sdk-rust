// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`BatchCreateBillScenarioCommitmentModification`](crate::operation::batch_create_bill_scenario_commitment_modification::builders::BatchCreateBillScenarioCommitmentModificationFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`bill_scenario_id(impl Into<String>)`](crate::operation::batch_create_bill_scenario_commitment_modification::builders::BatchCreateBillScenarioCommitmentModificationFluentBuilder::bill_scenario_id) / [`set_bill_scenario_id(Option<String>)`](crate::operation::batch_create_bill_scenario_commitment_modification::builders::BatchCreateBillScenarioCommitmentModificationFluentBuilder::set_bill_scenario_id):<br>required: **true**<br><p>The ID of the Bill Scenario for which you want to create the modeled commitment.</p><br>
    ///   - [`commitment_modifications(BatchCreateBillScenarioCommitmentModificationEntry)`](crate::operation::batch_create_bill_scenario_commitment_modification::builders::BatchCreateBillScenarioCommitmentModificationFluentBuilder::commitment_modifications) / [`set_commitment_modifications(Option<Vec::<BatchCreateBillScenarioCommitmentModificationEntry>>)`](crate::operation::batch_create_bill_scenario_commitment_modification::builders::BatchCreateBillScenarioCommitmentModificationFluentBuilder::set_commitment_modifications):<br>required: **true**<br><p>List of commitments that you want to model in the Bill Scenario.</p><br>
    ///   - [`client_token(impl Into<String>)`](crate::operation::batch_create_bill_scenario_commitment_modification::builders::BatchCreateBillScenarioCommitmentModificationFluentBuilder::client_token) / [`set_client_token(Option<String>)`](crate::operation::batch_create_bill_scenario_commitment_modification::builders::BatchCreateBillScenarioCommitmentModificationFluentBuilder::set_client_token):<br>required: **false**<br><p>A unique, case-sensitive identifier that you provide to ensure the idempotency of the request.</p><br>
    /// - On success, responds with [`BatchCreateBillScenarioCommitmentModificationOutput`](crate::operation::batch_create_bill_scenario_commitment_modification::BatchCreateBillScenarioCommitmentModificationOutput) with field(s):
    ///   - [`items(Option<Vec::<BatchCreateBillScenarioCommitmentModificationItem>>)`](crate::operation::batch_create_bill_scenario_commitment_modification::BatchCreateBillScenarioCommitmentModificationOutput::items): <p>Returns the list of successful commitment line items that were created for the Bill Scenario.</p>
    ///   - [`errors(Option<Vec::<BatchCreateBillScenarioCommitmentModificationError>>)`](crate::operation::batch_create_bill_scenario_commitment_modification::BatchCreateBillScenarioCommitmentModificationOutput::errors): <p>Returns the list of errors reason and the commitment item keys that cannot be created in the Bill Scenario.</p>
    /// - On failure, responds with [`SdkError<BatchCreateBillScenarioCommitmentModificationError>`](crate::operation::batch_create_bill_scenario_commitment_modification::BatchCreateBillScenarioCommitmentModificationError)
    pub fn batch_create_bill_scenario_commitment_modification(
        &self,
    ) -> crate::operation::batch_create_bill_scenario_commitment_modification::builders::BatchCreateBillScenarioCommitmentModificationFluentBuilder
    {
        crate::operation::batch_create_bill_scenario_commitment_modification::builders::BatchCreateBillScenarioCommitmentModificationFluentBuilder::new(self.handle.clone())
    }
}
