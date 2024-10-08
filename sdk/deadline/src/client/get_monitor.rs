// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetMonitor`](crate::operation::get_monitor::builders::GetMonitorFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`monitor_id(impl Into<String>)`](crate::operation::get_monitor::builders::GetMonitorFluentBuilder::monitor_id) / [`set_monitor_id(Option<String>)`](crate::operation::get_monitor::builders::GetMonitorFluentBuilder::set_monitor_id):<br>required: **true**<br><p>The unique identifier for the monitor. This ID is returned by the <code>CreateMonitor</code> operation.</p><br>
    /// - On success, responds with [`GetMonitorOutput`](crate::operation::get_monitor::GetMonitorOutput) with field(s):
    ///   - [`monitor_id(String)`](crate::operation::get_monitor::GetMonitorOutput::monitor_id): <p>The unique identifier for the monitor.</p>
    ///   - [`display_name(String)`](crate::operation::get_monitor::GetMonitorOutput::display_name): <p>The name used to identify the monitor on the Deadline Cloud console.</p><important>  <p>This field can store any content. Escape or encode this content before displaying it on a webpage or any other system that might interpret the content of this field.</p> </important>
    ///   - [`subdomain(String)`](crate::operation::get_monitor::GetMonitorOutput::subdomain): <p>The subdomain used for the monitor URL. The full URL of the monitor is subdomain.Region.deadlinecloud.amazonaws.com.</p>
    ///   - [`url(String)`](crate::operation::get_monitor::GetMonitorOutput::url): <p>The complete URL of the monitor. The full URL of the monitor is subdomain.Region.deadlinecloud.amazonaws.com.</p>
    ///   - [`role_arn(String)`](crate::operation::get_monitor::GetMonitorOutput::role_arn): <p>The Amazon Resource Name (ARN) of the IAM role for the monitor. Users of the monitor use this role to access Deadline Cloud resources.</p>
    ///   - [`identity_center_instance_arn(String)`](crate::operation::get_monitor::GetMonitorOutput::identity_center_instance_arn): <p>The Amazon Resource Name (ARN) of the IAM Identity Center instance responsible for authenticating monitor users.</p>
    ///   - [`identity_center_application_arn(String)`](crate::operation::get_monitor::GetMonitorOutput::identity_center_application_arn): <p>The Amazon Resource Name (ARN) that the IAM Identity Center assigned to the monitor when it was created.</p>
    ///   - [`created_at(DateTime)`](crate::operation::get_monitor::GetMonitorOutput::created_at): <p>The UNIX timestamp of the date and time that the monitor was created.</p>
    ///   - [`created_by(String)`](crate::operation::get_monitor::GetMonitorOutput::created_by): <p>The user name of the person that created the monitor.</p>
    ///   - [`updated_at(Option<DateTime>)`](crate::operation::get_monitor::GetMonitorOutput::updated_at): <p>The UNIX timestamp of the last date and time that the monitor was updated.</p>
    ///   - [`updated_by(Option<String>)`](crate::operation::get_monitor::GetMonitorOutput::updated_by): <p>The user name of the person that last updated the monitor.</p>
    /// - On failure, responds with [`SdkError<GetMonitorError>`](crate::operation::get_monitor::GetMonitorError)
    pub fn get_monitor(&self) -> crate::operation::get_monitor::builders::GetMonitorFluentBuilder {
        crate::operation::get_monitor::builders::GetMonitorFluentBuilder::new(self.handle.clone())
    }
}
