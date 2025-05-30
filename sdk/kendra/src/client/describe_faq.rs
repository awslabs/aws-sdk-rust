// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DescribeFaq`](crate::operation::describe_faq::builders::DescribeFaqFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`id(impl Into<String>)`](crate::operation::describe_faq::builders::DescribeFaqFluentBuilder::id) / [`set_id(Option<String>)`](crate::operation::describe_faq::builders::DescribeFaqFluentBuilder::set_id):<br>required: **true**<br><p>The identifier of the FAQ you want to get information on.</p><br>
    ///   - [`index_id(impl Into<String>)`](crate::operation::describe_faq::builders::DescribeFaqFluentBuilder::index_id) / [`set_index_id(Option<String>)`](crate::operation::describe_faq::builders::DescribeFaqFluentBuilder::set_index_id):<br>required: **true**<br><p>The identifier of the index for the FAQ.</p><br>
    /// - On success, responds with [`DescribeFaqOutput`](crate::operation::describe_faq::DescribeFaqOutput) with field(s):
    ///   - [`id(Option<String>)`](crate::operation::describe_faq::DescribeFaqOutput::id): <p>The identifier of the FAQ.</p>
    ///   - [`index_id(Option<String>)`](crate::operation::describe_faq::DescribeFaqOutput::index_id): <p>The identifier of the index for the FAQ.</p>
    ///   - [`name(Option<String>)`](crate::operation::describe_faq::DescribeFaqOutput::name): <p>The name that you gave the FAQ when it was created.</p>
    ///   - [`description(Option<String>)`](crate::operation::describe_faq::DescribeFaqOutput::description): <p>The description of the FAQ that you provided when it was created.</p>
    ///   - [`created_at(Option<DateTime>)`](crate::operation::describe_faq::DescribeFaqOutput::created_at): <p>The Unix timestamp when the FAQ was created.</p>
    ///   - [`updated_at(Option<DateTime>)`](crate::operation::describe_faq::DescribeFaqOutput::updated_at): <p>The Unix timestamp when the FAQ was last updated.</p>
    ///   - [`s3_path(Option<S3Path>)`](crate::operation::describe_faq::DescribeFaqOutput::s3_path): <p>Information required to find a specific file in an Amazon S3 bucket.</p>
    ///   - [`status(Option<FaqStatus>)`](crate::operation::describe_faq::DescribeFaqOutput::status): <p>The status of the FAQ. It is ready to use when the status is <code>ACTIVE</code>.</p>
    ///   - [`role_arn(Option<String>)`](crate::operation::describe_faq::DescribeFaqOutput::role_arn): <p>The Amazon Resource Name (ARN) of the IAM role that provides access to the S3 bucket containing the FAQ file.</p>
    ///   - [`error_message(Option<String>)`](crate::operation::describe_faq::DescribeFaqOutput::error_message): <p>If the <code>Status</code> field is <code>FAILED</code>, the <code>ErrorMessage</code> field contains the reason why the FAQ failed.</p>
    ///   - [`file_format(Option<FaqFileFormat>)`](crate::operation::describe_faq::DescribeFaqOutput::file_format): <p>The file format used for the FAQ file.</p>
    ///   - [`language_code(Option<String>)`](crate::operation::describe_faq::DescribeFaqOutput::language_code): <p>The code for a language. This shows a supported language for the FAQ document. English is supported by default. For more information on supported languages, including their codes, see <a href="https://docs.aws.amazon.com/kendra/latest/dg/in-adding-languages.html">Adding documents in languages other than English</a>.</p>
    /// - On failure, responds with [`SdkError<DescribeFaqError>`](crate::operation::describe_faq::DescribeFaqError)
    pub fn describe_faq(&self) -> crate::operation::describe_faq::builders::DescribeFaqFluentBuilder {
        crate::operation::describe_faq::builders::DescribeFaqFluentBuilder::new(self.handle.clone())
    }
}
