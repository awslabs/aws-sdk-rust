// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListMonitors`](crate::operation::list_monitors::builders::ListMonitorsFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_monitors::builders::ListMonitorsFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_monitors::builders::ListMonitorsFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_monitors::builders::ListMonitorsFluentBuilder::set_next_token):<br>required: **false**<br><p>The token for the next set of results. You receive this token from a previous call.</p><br>
    ///   - [`max_results(i32)`](crate::operation::list_monitors::builders::ListMonitorsFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_monitors::builders::ListMonitorsFluentBuilder::set_max_results):<br>required: **false**<br><p>The number of query results that you want to return with this call.</p><br>
    ///   - [`monitor_status(MonitorStatus)`](crate::operation::list_monitors::builders::ListMonitorsFluentBuilder::monitor_status) / [`set_monitor_status(Option<MonitorStatus>)`](crate::operation::list_monitors::builders::ListMonitorsFluentBuilder::set_monitor_status):<br>required: **false**<br><p>The status of a monitor. The status can be one of the following</p> <ul>  <li>   <p><code>PENDING</code>: The monitor is in the process of being created.</p></li>  <li>   <p><code>ACTIVE</code>: The monitor is active.</p></li>  <li>   <p><code>INACTIVE</code>: The monitor is inactive.</p></li>  <li>   <p><code>ERROR</code>: Monitor creation failed due to an error.</p></li>  <li>   <p><code>DELETING</code>: The monitor is in the process of being deleted.</p></li> </ul><br>
    /// - On success, responds with [`ListMonitorsOutput`](crate::operation::list_monitors::ListMonitorsOutput) with field(s):
    ///   - [`monitors(Vec::<MonitorSummary>)`](crate::operation::list_monitors::ListMonitorsOutput::monitors): <p>The monitors that are in an account.</p>
    ///   - [`next_token(Option<String>)`](crate::operation::list_monitors::ListMonitorsOutput::next_token): <p>The token for the next set of results. You receive this token from a previous call.</p>
    /// - On failure, responds with [`SdkError<ListMonitorsError>`](crate::operation::list_monitors::ListMonitorsError)
    pub fn list_monitors(&self) -> crate::operation::list_monitors::builders::ListMonitorsFluentBuilder {
        crate::operation::list_monitors::builders::ListMonitorsFluentBuilder::new(self.handle.clone())
    }
}
