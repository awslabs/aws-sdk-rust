// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`CreateSampleFindings`](crate::operation::create_sample_findings::builders::CreateSampleFindingsFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`detector_id(impl Into<String>)`](crate::operation::create_sample_findings::builders::CreateSampleFindingsFluentBuilder::detector_id) / [`set_detector_id(Option<String>)`](crate::operation::create_sample_findings::builders::CreateSampleFindingsFluentBuilder::set_detector_id):<br>required: **true**<br><p>The ID of the detector for which you need to create sample findings.</p> <p>To find the <code>detectorId</code> in the current Region, see the Settings page in the GuardDuty console, or run the <a href="https://docs.aws.amazon.com/guardduty/latest/APIReference/API_ListDetectors.html">ListDetectors</a> API.</p><br>
    ///   - [`finding_types(impl Into<String>)`](crate::operation::create_sample_findings::builders::CreateSampleFindingsFluentBuilder::finding_types) / [`set_finding_types(Option<Vec::<String>>)`](crate::operation::create_sample_findings::builders::CreateSampleFindingsFluentBuilder::set_finding_types):<br>required: **false**<br><p>The types of sample findings to generate.</p><br>
    /// - On success, responds with [`CreateSampleFindingsOutput`](crate::operation::create_sample_findings::CreateSampleFindingsOutput)
    /// - On failure, responds with [`SdkError<CreateSampleFindingsError>`](crate::operation::create_sample_findings::CreateSampleFindingsError)
    pub fn create_sample_findings(&self) -> crate::operation::create_sample_findings::builders::CreateSampleFindingsFluentBuilder {
        crate::operation::create_sample_findings::builders::CreateSampleFindingsFluentBuilder::new(self.handle.clone())
    }
}
