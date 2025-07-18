// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`CreateIndex`](crate::operation::create_index::builders::CreateIndexFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`vector_bucket_name(impl Into<String>)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::vector_bucket_name) / [`set_vector_bucket_name(Option<String>)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::set_vector_bucket_name):<br>required: **false**<br><p>The name of the vector bucket to create the vector index in.</p><br>
    ///   - [`vector_bucket_arn(impl Into<String>)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::vector_bucket_arn) / [`set_vector_bucket_arn(Option<String>)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::set_vector_bucket_arn):<br>required: **false**<br><p>The Amazon Resource Name (ARN) of the vector bucket to create the vector index in.</p><br>
    ///   - [`index_name(impl Into<String>)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::index_name) / [`set_index_name(Option<String>)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::set_index_name):<br>required: **true**<br><p>The name of the vector index to create.</p><br>
    ///   - [`data_type(DataType)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::data_type) / [`set_data_type(Option<DataType>)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::set_data_type):<br>required: **true**<br><p>The data type of the vectors to be inserted into the vector index.</p><br>
    ///   - [`dimension(i32)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::dimension) / [`set_dimension(Option<i32>)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::set_dimension):<br>required: **true**<br><p>The dimensions of the vectors to be inserted into the vector index.</p><br>
    ///   - [`distance_metric(DistanceMetric)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::distance_metric) / [`set_distance_metric(Option<DistanceMetric>)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::set_distance_metric):<br>required: **true**<br><p>The distance metric to be used for similarity search.</p><br>
    ///   - [`metadata_configuration(MetadataConfiguration)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::metadata_configuration) / [`set_metadata_configuration(Option<MetadataConfiguration>)`](crate::operation::create_index::builders::CreateIndexFluentBuilder::set_metadata_configuration):<br>required: **false**<br><p>The metadata configuration for the vector index.</p><br>
    /// - On success, responds with [`CreateIndexOutput`](crate::operation::create_index::CreateIndexOutput)
    /// - On failure, responds with [`SdkError<CreateIndexError>`](crate::operation::create_index::CreateIndexError)
    pub fn create_index(&self) -> crate::operation::create_index::builders::CreateIndexFluentBuilder {
        crate::operation::create_index::builders::CreateIndexFluentBuilder::new(self.handle.clone())
    }
}
