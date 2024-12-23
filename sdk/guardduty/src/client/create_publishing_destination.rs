// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`CreatePublishingDestination`](crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`detector_id(impl Into<String>)`](crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder::detector_id) / [`set_detector_id(Option<String>)`](crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder::set_detector_id):<br>required: **true**<br><p>The ID of the GuardDuty detector associated with the publishing destination.</p> <p>To find the <code>detectorId</code> in the current Region, see the Settings page in the GuardDuty console, or run the <a href="https://docs.aws.amazon.com/guardduty/latest/APIReference/API_ListDetectors.html">ListDetectors</a> API.</p><br>
    ///   - [`destination_type(DestinationType)`](crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder::destination_type) / [`set_destination_type(Option<DestinationType>)`](crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder::set_destination_type):<br>required: **true**<br><p>The type of resource for the publishing destination. Currently only Amazon S3 buckets are supported.</p><br>
    ///   - [`destination_properties(DestinationProperties)`](crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder::destination_properties) / [`set_destination_properties(Option<DestinationProperties>)`](crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder::set_destination_properties):<br>required: **true**<br><p>The properties of the publishing destination, including the ARNs for the destination and the KMS key used for encryption.</p><br>
    ///   - [`client_token(impl Into<String>)`](crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder::client_token) / [`set_client_token(Option<String>)`](crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder::set_client_token):<br>required: **false**<br><p>The idempotency token for the request.</p><br>
    /// - On success, responds with [`CreatePublishingDestinationOutput`](crate::operation::create_publishing_destination::CreatePublishingDestinationOutput) with field(s):
    ///   - [`destination_id(Option<String>)`](crate::operation::create_publishing_destination::CreatePublishingDestinationOutput::destination_id): <p>The ID of the publishing destination that is created.</p>
    /// - On failure, responds with [`SdkError<CreatePublishingDestinationError>`](crate::operation::create_publishing_destination::CreatePublishingDestinationError)
    pub fn create_publishing_destination(
        &self,
    ) -> crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder {
        crate::operation::create_publishing_destination::builders::CreatePublishingDestinationFluentBuilder::new(self.handle.clone())
    }
}
