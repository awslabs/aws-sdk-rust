// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetTable`](crate::operation::get_table::builders::GetTableFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`keyspace_name(impl Into<String>)`](crate::operation::get_table::builders::GetTableFluentBuilder::keyspace_name) / [`set_keyspace_name(Option<String>)`](crate::operation::get_table::builders::GetTableFluentBuilder::set_keyspace_name):<br>required: **true**<br><p>The name of the keyspace that the table is stored in.</p><br>
    ///   - [`table_name(impl Into<String>)`](crate::operation::get_table::builders::GetTableFluentBuilder::table_name) / [`set_table_name(Option<String>)`](crate::operation::get_table::builders::GetTableFluentBuilder::set_table_name):<br>required: **true**<br><p>The name of the table.</p><br>
    /// - On success, responds with [`GetTableOutput`](crate::operation::get_table::GetTableOutput) with field(s):
    ///   - [`keyspace_name(String)`](crate::operation::get_table::GetTableOutput::keyspace_name): <p>The name of the keyspace that the specified table is stored in.</p>
    ///   - [`table_name(String)`](crate::operation::get_table::GetTableOutput::table_name): <p>The name of the specified table.</p>
    ///   - [`resource_arn(String)`](crate::operation::get_table::GetTableOutput::resource_arn): <p>The Amazon Resource Name (ARN) of the specified table.</p>
    ///   - [`creation_timestamp(Option<DateTime>)`](crate::operation::get_table::GetTableOutput::creation_timestamp): <p>The creation timestamp of the specified table.</p>
    ///   - [`status(Option<TableStatus>)`](crate::operation::get_table::GetTableOutput::status): <p>The current status of the specified table.</p>
    ///   - [`schema_definition(Option<SchemaDefinition>)`](crate::operation::get_table::GetTableOutput::schema_definition): <p>The schema definition of the specified table.</p>
    ///   - [`capacity_specification(Option<CapacitySpecificationSummary>)`](crate::operation::get_table::GetTableOutput::capacity_specification): <p>The read/write throughput capacity mode for a table. The options are:</p> <ul>  <li>   <p><code>throughputMode:PAY_PER_REQUEST</code></p></li>  <li>   <p><code>throughputMode:PROVISIONED</code></p></li> </ul>
    ///   - [`encryption_specification(Option<EncryptionSpecification>)`](crate::operation::get_table::GetTableOutput::encryption_specification): <p>The encryption settings of the specified table.</p>
    ///   - [`point_in_time_recovery(Option<PointInTimeRecoverySummary>)`](crate::operation::get_table::GetTableOutput::point_in_time_recovery): <p>The point-in-time recovery status of the specified table.</p>
    ///   - [`ttl(Option<TimeToLive>)`](crate::operation::get_table::GetTableOutput::ttl): <p>The custom Time to Live settings of the specified table.</p>
    ///   - [`default_time_to_live(Option<i32>)`](crate::operation::get_table::GetTableOutput::default_time_to_live): <p>The default Time to Live settings in seconds of the specified table.</p>
    ///   - [`comment(Option<Comment>)`](crate::operation::get_table::GetTableOutput::comment): <p>The the description of the specified table.</p>
    ///   - [`client_side_timestamps(Option<ClientSideTimestamps>)`](crate::operation::get_table::GetTableOutput::client_side_timestamps): <p>The client-side timestamps setting of the table.</p>
    ///   - [`replica_specifications(Option<Vec::<ReplicaSpecificationSummary>>)`](crate::operation::get_table::GetTableOutput::replica_specifications): <p>Returns the Amazon Web Services Region specific settings of all Regions a multi-Region table is replicated in.</p>
    ///   - [`latest_stream_arn(Option<String>)`](crate::operation::get_table::GetTableOutput::latest_stream_arn): <p>The Amazon Resource Name (ARN) of the stream.</p>
    ///   - [`cdc_specification(Option<CdcSpecificationSummary>)`](crate::operation::get_table::GetTableOutput::cdc_specification): <p>The CDC stream settings of the table.</p>
    /// - On failure, responds with [`SdkError<GetTableError>`](crate::operation::get_table::GetTableError)
    pub fn get_table(&self) -> crate::operation::get_table::builders::GetTableFluentBuilder {
        crate::operation::get_table::builders::GetTableFluentBuilder::new(self.handle.clone())
    }
}
