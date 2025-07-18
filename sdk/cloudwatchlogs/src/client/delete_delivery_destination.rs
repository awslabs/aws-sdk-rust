// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DeleteDeliveryDestination`](crate::operation::delete_delivery_destination::builders::DeleteDeliveryDestinationFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`name(impl Into<String>)`](crate::operation::delete_delivery_destination::builders::DeleteDeliveryDestinationFluentBuilder::name) / [`set_name(Option<String>)`](crate::operation::delete_delivery_destination::builders::DeleteDeliveryDestinationFluentBuilder::set_name):<br>required: **true**<br><p>The name of the delivery destination that you want to delete. You can find a list of delivery destination names by using the <a href="https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_DescribeDeliveryDestinations.html">DescribeDeliveryDestinations</a> operation.</p><br>
    /// - On success, responds with [`DeleteDeliveryDestinationOutput`](crate::operation::delete_delivery_destination::DeleteDeliveryDestinationOutput)
    /// - On failure, responds with [`SdkError<DeleteDeliveryDestinationError>`](crate::operation::delete_delivery_destination::DeleteDeliveryDestinationError)
    pub fn delete_delivery_destination(&self) -> crate::operation::delete_delivery_destination::builders::DeleteDeliveryDestinationFluentBuilder {
        crate::operation::delete_delivery_destination::builders::DeleteDeliveryDestinationFluentBuilder::new(self.handle.clone())
    }
}
