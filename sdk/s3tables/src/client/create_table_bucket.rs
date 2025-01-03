// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`CreateTableBucket`](crate::operation::create_table_bucket::builders::CreateTableBucketFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`name(impl Into<String>)`](crate::operation::create_table_bucket::builders::CreateTableBucketFluentBuilder::name) / [`set_name(Option<String>)`](crate::operation::create_table_bucket::builders::CreateTableBucketFluentBuilder::set_name):<br>required: **true**<br><p>The name for the table bucket.</p><br>
    /// - On success, responds with [`CreateTableBucketOutput`](crate::operation::create_table_bucket::CreateTableBucketOutput) with field(s):
    ///   - [`arn(String)`](crate::operation::create_table_bucket::CreateTableBucketOutput::arn): <p>The Amazon Resource Name (ARN) of the table bucket.</p>
    /// - On failure, responds with [`SdkError<CreateTableBucketError>`](crate::operation::create_table_bucket::CreateTableBucketError)
    pub fn create_table_bucket(&self) -> crate::operation::create_table_bucket::builders::CreateTableBucketFluentBuilder {
        crate::operation::create_table_bucket::builders::CreateTableBucketFluentBuilder::new(self.handle.clone())
    }
}
