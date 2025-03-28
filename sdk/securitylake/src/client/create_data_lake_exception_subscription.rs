// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`CreateDataLakeExceptionSubscription`](crate::operation::create_data_lake_exception_subscription::builders::CreateDataLakeExceptionSubscriptionFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`subscription_protocol(impl Into<String>)`](crate::operation::create_data_lake_exception_subscription::builders::CreateDataLakeExceptionSubscriptionFluentBuilder::subscription_protocol) / [`set_subscription_protocol(Option<String>)`](crate::operation::create_data_lake_exception_subscription::builders::CreateDataLakeExceptionSubscriptionFluentBuilder::set_subscription_protocol):<br>required: **true**<br><p>The subscription protocol to which exception notifications are posted.</p><br>
    ///   - [`notification_endpoint(impl Into<String>)`](crate::operation::create_data_lake_exception_subscription::builders::CreateDataLakeExceptionSubscriptionFluentBuilder::notification_endpoint) / [`set_notification_endpoint(Option<String>)`](crate::operation::create_data_lake_exception_subscription::builders::CreateDataLakeExceptionSubscriptionFluentBuilder::set_notification_endpoint):<br>required: **true**<br><p>The Amazon Web Services account where you want to receive exception notifications.</p><br>
    ///   - [`exception_time_to_live(i64)`](crate::operation::create_data_lake_exception_subscription::builders::CreateDataLakeExceptionSubscriptionFluentBuilder::exception_time_to_live) / [`set_exception_time_to_live(Option<i64>)`](crate::operation::create_data_lake_exception_subscription::builders::CreateDataLakeExceptionSubscriptionFluentBuilder::set_exception_time_to_live):<br>required: **false**<br><p>The expiration period and time-to-live (TTL). It is the duration of time until which the exception message remains.</p><br>
    /// - On success, responds with [`CreateDataLakeExceptionSubscriptionOutput`](crate::operation::create_data_lake_exception_subscription::CreateDataLakeExceptionSubscriptionOutput)
    /// - On failure, responds with [`SdkError<CreateDataLakeExceptionSubscriptionError>`](crate::operation::create_data_lake_exception_subscription::CreateDataLakeExceptionSubscriptionError)
    pub fn create_data_lake_exception_subscription(
        &self,
    ) -> crate::operation::create_data_lake_exception_subscription::builders::CreateDataLakeExceptionSubscriptionFluentBuilder {
        crate::operation::create_data_lake_exception_subscription::builders::CreateDataLakeExceptionSubscriptionFluentBuilder::new(
            self.handle.clone(),
        )
    }
}
