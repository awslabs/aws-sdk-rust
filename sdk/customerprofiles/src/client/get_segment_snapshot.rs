// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetSegmentSnapshot`](crate::operation::get_segment_snapshot::builders::GetSegmentSnapshotFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`domain_name(impl Into<String>)`](crate::operation::get_segment_snapshot::builders::GetSegmentSnapshotFluentBuilder::domain_name) / [`set_domain_name(Option<String>)`](crate::operation::get_segment_snapshot::builders::GetSegmentSnapshotFluentBuilder::set_domain_name):<br>required: **true**<br><p>The unique identifier of the domain.</p><br>
    ///   - [`segment_definition_name(impl Into<String>)`](crate::operation::get_segment_snapshot::builders::GetSegmentSnapshotFluentBuilder::segment_definition_name) / [`set_segment_definition_name(Option<String>)`](crate::operation::get_segment_snapshot::builders::GetSegmentSnapshotFluentBuilder::set_segment_definition_name):<br>required: **true**<br><p>The unique name of the segment definition.</p><br>
    ///   - [`snapshot_id(impl Into<String>)`](crate::operation::get_segment_snapshot::builders::GetSegmentSnapshotFluentBuilder::snapshot_id) / [`set_snapshot_id(Option<String>)`](crate::operation::get_segment_snapshot::builders::GetSegmentSnapshotFluentBuilder::set_snapshot_id):<br>required: **true**<br><p>The unique identifier of the segment snapshot.</p><br>
    /// - On success, responds with [`GetSegmentSnapshotOutput`](crate::operation::get_segment_snapshot::GetSegmentSnapshotOutput) with field(s):
    ///   - [`snapshot_id(String)`](crate::operation::get_segment_snapshot::GetSegmentSnapshotOutput::snapshot_id): <p>The unique identifier of the segment snapshot.</p>
    ///   - [`status(SegmentSnapshotStatus)`](crate::operation::get_segment_snapshot::GetSegmentSnapshotOutput::status): <p>The status of the asynchronous job for exporting the segment snapshot.</p>
    ///   - [`status_message(Option<String>)`](crate::operation::get_segment_snapshot::GetSegmentSnapshotOutput::status_message): <p>The status message of the asynchronous job for exporting the segment snapshot.</p>
    ///   - [`data_format(DataFormat)`](crate::operation::get_segment_snapshot::GetSegmentSnapshotOutput::data_format): <p>The format in which the segment will be exported.</p>
    ///   - [`encryption_key(Option<String>)`](crate::operation::get_segment_snapshot::GetSegmentSnapshotOutput::encryption_key): <p>The Amazon Resource Name (ARN) of the KMS key used to encrypt the exported segment.</p>
    ///   - [`role_arn(Option<String>)`](crate::operation::get_segment_snapshot::GetSegmentSnapshotOutput::role_arn): <p>The Amazon Resource Name (ARN) of the IAM role that allows Customer Profiles service principal to assume the role for conducting KMS and S3 operations.</p>
    ///   - [`destination_uri(Option<String>)`](crate::operation::get_segment_snapshot::GetSegmentSnapshotOutput::destination_uri): <p>The destination to which the segment will be exported. This field must be provided if the request is not submitted from the Amazon Connect Admin Website.</p>
    /// - On failure, responds with [`SdkError<GetSegmentSnapshotError>`](crate::operation::get_segment_snapshot::GetSegmentSnapshotError)
    pub fn get_segment_snapshot(&self) -> crate::operation::get_segment_snapshot::builders::GetSegmentSnapshotFluentBuilder {
        crate::operation::get_segment_snapshot::builders::GetSegmentSnapshotFluentBuilder::new(self.handle.clone())
    }
}
