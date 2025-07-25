// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetApplication`](crate::operation::get_application::builders::GetApplicationFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`arn(impl Into<String>)`](crate::operation::get_application::builders::GetApplicationFluentBuilder::arn) / [`set_arn(Option<String>)`](crate::operation::get_application::builders::GetApplicationFluentBuilder::set_arn):<br>required: **true**<br><p>The Amazon Resource Name (ARN) of the Application.</p><br>
    /// - On success, responds with [`GetApplicationOutput`](crate::operation::get_application::GetApplicationOutput) with field(s):
    ///   - [`arn(Option<String>)`](crate::operation::get_application::GetApplicationOutput::arn): <p>The Amazon Resource Name (ARN) of the Application.</p>
    ///   - [`id(Option<String>)`](crate::operation::get_application::GetApplicationOutput::id): <p>A unique identifier for the Application.</p>
    ///   - [`name(Option<String>)`](crate::operation::get_application::GetApplicationOutput::name): <p>The name of the application.</p>
    ///   - [`namespace(Option<String>)`](crate::operation::get_application::GetApplicationOutput::namespace): <p>The namespace of the application.</p>
    ///   - [`description(Option<String>)`](crate::operation::get_application::GetApplicationOutput::description): <p>The description of the application.</p>
    ///   - [`application_source_config(Option<ApplicationSourceConfig>)`](crate::operation::get_application::GetApplicationOutput::application_source_config): <p>The configuration for where the application should be loaded from.</p>
    ///   - [`subscriptions(Option<Vec::<Subscription>>)`](crate::operation::get_application::GetApplicationOutput::subscriptions): <p>The events that the application subscribes.</p>
    ///   - [`publications(Option<Vec::<Publication>>)`](crate::operation::get_application::GetApplicationOutput::publications): <p>The events that the application publishes.</p>
    ///   - [`created_time(Option<DateTime>)`](crate::operation::get_application::GetApplicationOutput::created_time): <p>The created time of the Application.</p>
    ///   - [`last_modified_time(Option<DateTime>)`](crate::operation::get_application::GetApplicationOutput::last_modified_time): <p>The last modified time of the Application.</p>
    ///   - [`tags(Option<HashMap::<String, String>>)`](crate::operation::get_application::GetApplicationOutput::tags): <p>The tags used to organize, track, or control access for this resource. For example, { "tags": {"key1":"value1", "key2":"value2"} }.</p>
    ///   - [`permissions(Option<Vec::<String>>)`](crate::operation::get_application::GetApplicationOutput::permissions): <p>The configuration of events or requests that the application has access to.</p>
    ///   - [`is_service(bool)`](crate::operation::get_application::GetApplicationOutput::is_service): <p>Indicates whether the application is a service.</p>
    ///   - [`initialization_timeout(Option<i32>)`](crate::operation::get_application::GetApplicationOutput::initialization_timeout): <p>The maximum time in milliseconds allowed to establish a connection with the workspace.</p>
    ///   - [`application_config(Option<ApplicationConfig>)`](crate::operation::get_application::GetApplicationOutput::application_config): <p>The configuration settings for the application.</p>
    ///   - [`iframe_config(Option<IframeConfig>)`](crate::operation::get_application::GetApplicationOutput::iframe_config): <p>The iframe configuration for the application.</p>
    /// - On failure, responds with [`SdkError<GetApplicationError>`](crate::operation::get_application::GetApplicationError)
    pub fn get_application(&self) -> crate::operation::get_application::builders::GetApplicationFluentBuilder {
        crate::operation::get_application::builders::GetApplicationFluentBuilder::new(self.handle.clone())
    }
}
