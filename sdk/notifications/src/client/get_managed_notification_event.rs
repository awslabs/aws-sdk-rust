// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetManagedNotificationEvent`](crate::operation::get_managed_notification_event::builders::GetManagedNotificationEventFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`arn(impl Into<String>)`](crate::operation::get_managed_notification_event::builders::GetManagedNotificationEventFluentBuilder::arn) / [`set_arn(Option<String>)`](crate::operation::get_managed_notification_event::builders::GetManagedNotificationEventFluentBuilder::set_arn):<br>required: **true**<br><p>The Amazon Resource Name (ARN) of the <code>ManagedNotificationEvent</code> to return.</p><br>
    ///   - [`locale(LocaleCode)`](crate::operation::get_managed_notification_event::builders::GetManagedNotificationEventFluentBuilder::locale) / [`set_locale(Option<LocaleCode>)`](crate::operation::get_managed_notification_event::builders::GetManagedNotificationEventFluentBuilder::set_locale):<br>required: **false**<br><p>The locale code of the language used for the retrieved <code>ManagedNotificationEvent</code>. The default locale is English <code>(en_US)</code>.</p><br>
    /// - On success, responds with [`GetManagedNotificationEventOutput`](crate::operation::get_managed_notification_event::GetManagedNotificationEventOutput) with field(s):
    ///   - [`arn(String)`](crate::operation::get_managed_notification_event::GetManagedNotificationEventOutput::arn): <p>The ARN of the resource.</p>
    ///   - [`managed_notification_configuration_arn(String)`](crate::operation::get_managed_notification_event::GetManagedNotificationEventOutput::managed_notification_configuration_arn): <p>The ARN of the <code>ManagedNotificationConfiguration</code>.</p>
    ///   - [`creation_time(DateTime)`](crate::operation::get_managed_notification_event::GetManagedNotificationEventOutput::creation_time): <p>The creation time of the <code>ManagedNotificationEvent</code>.</p>
    ///   - [`content(Option<ManagedNotificationEvent>)`](crate::operation::get_managed_notification_event::GetManagedNotificationEventOutput::content): <p>The content of the <code>ManagedNotificationEvent</code>.</p>
    /// - On failure, responds with [`SdkError<GetManagedNotificationEventError>`](crate::operation::get_managed_notification_event::GetManagedNotificationEventError)
    pub fn get_managed_notification_event(
        &self,
    ) -> crate::operation::get_managed_notification_event::builders::GetManagedNotificationEventFluentBuilder {
        crate::operation::get_managed_notification_event::builders::GetManagedNotificationEventFluentBuilder::new(self.handle.clone())
    }
}
