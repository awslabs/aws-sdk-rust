// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetTableBucketMaintenanceConfiguration`](crate::operation::get_table_bucket_maintenance_configuration::builders::GetTableBucketMaintenanceConfigurationFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`table_bucket_arn(impl Into<String>)`](crate::operation::get_table_bucket_maintenance_configuration::builders::GetTableBucketMaintenanceConfigurationFluentBuilder::table_bucket_arn) / [`set_table_bucket_arn(Option<String>)`](crate::operation::get_table_bucket_maintenance_configuration::builders::GetTableBucketMaintenanceConfigurationFluentBuilder::set_table_bucket_arn):<br>required: **true**<br><p>The Amazon Resource Name (ARN) of the table bucket associated with the maintenance configuration.</p><br>
    /// - On success, responds with [`GetTableBucketMaintenanceConfigurationOutput`](crate::operation::get_table_bucket_maintenance_configuration::GetTableBucketMaintenanceConfigurationOutput) with field(s):
    ///   - [`table_bucket_arn(String)`](crate::operation::get_table_bucket_maintenance_configuration::GetTableBucketMaintenanceConfigurationOutput::table_bucket_arn): <p>The Amazon Resource Name (ARN) of the table bucket associated with the maintenance configuration.</p>
    ///   - [`configuration(HashMap::<TableBucketMaintenanceType, TableBucketMaintenanceConfigurationValue>)`](crate::operation::get_table_bucket_maintenance_configuration::GetTableBucketMaintenanceConfigurationOutput::configuration): <p>Details about the maintenance configuration for the table bucket.</p>
    /// - On failure, responds with [`SdkError<GetTableBucketMaintenanceConfigurationError>`](crate::operation::get_table_bucket_maintenance_configuration::GetTableBucketMaintenanceConfigurationError)
    pub fn get_table_bucket_maintenance_configuration(
        &self,
    ) -> crate::operation::get_table_bucket_maintenance_configuration::builders::GetTableBucketMaintenanceConfigurationFluentBuilder {
        crate::operation::get_table_bucket_maintenance_configuration::builders::GetTableBucketMaintenanceConfigurationFluentBuilder::new(
            self.handle.clone(),
        )
    }
}
