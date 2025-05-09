// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`CreatePartnerAppPresignedUrl`](crate::operation::create_partner_app_presigned_url::builders::CreatePartnerAppPresignedUrlFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`arn(impl Into<String>)`](crate::operation::create_partner_app_presigned_url::builders::CreatePartnerAppPresignedUrlFluentBuilder::arn) / [`set_arn(Option<String>)`](crate::operation::create_partner_app_presigned_url::builders::CreatePartnerAppPresignedUrlFluentBuilder::set_arn):<br>required: **true**<br><p>The ARN of the SageMaker Partner AI App to create the presigned URL for.</p><br>
    ///   - [`expires_in_seconds(i32)`](crate::operation::create_partner_app_presigned_url::builders::CreatePartnerAppPresignedUrlFluentBuilder::expires_in_seconds) / [`set_expires_in_seconds(Option<i32>)`](crate::operation::create_partner_app_presigned_url::builders::CreatePartnerAppPresignedUrlFluentBuilder::set_expires_in_seconds):<br>required: **false**<br><p>The time that will pass before the presigned URL expires.</p><br>
    ///   - [`session_expiration_duration_in_seconds(i32)`](crate::operation::create_partner_app_presigned_url::builders::CreatePartnerAppPresignedUrlFluentBuilder::session_expiration_duration_in_seconds) / [`set_session_expiration_duration_in_seconds(Option<i32>)`](crate::operation::create_partner_app_presigned_url::builders::CreatePartnerAppPresignedUrlFluentBuilder::set_session_expiration_duration_in_seconds):<br>required: **false**<br><p>Indicates how long the Amazon SageMaker Partner AI App session can be accessed for after logging in.</p><br>
    /// - On success, responds with [`CreatePartnerAppPresignedUrlOutput`](crate::operation::create_partner_app_presigned_url::CreatePartnerAppPresignedUrlOutput) with field(s):
    ///   - [`url(Option<String>)`](crate::operation::create_partner_app_presigned_url::CreatePartnerAppPresignedUrlOutput::url): <p>The presigned URL that you can use to access the SageMaker Partner AI App.</p>
    /// - On failure, responds with [`SdkError<CreatePartnerAppPresignedUrlError>`](crate::operation::create_partner_app_presigned_url::CreatePartnerAppPresignedUrlError)
    pub fn create_partner_app_presigned_url(
        &self,
    ) -> crate::operation::create_partner_app_presigned_url::builders::CreatePartnerAppPresignedUrlFluentBuilder {
        crate::operation::create_partner_app_presigned_url::builders::CreatePartnerAppPresignedUrlFluentBuilder::new(self.handle.clone())
    }
}
