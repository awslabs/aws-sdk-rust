// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::start_dashboard_snapshot_job_schedule::_start_dashboard_snapshot_job_schedule_output::StartDashboardSnapshotJobScheduleOutputBuilder;

pub use crate::operation::start_dashboard_snapshot_job_schedule::_start_dashboard_snapshot_job_schedule_input::StartDashboardSnapshotJobScheduleInputBuilder;

impl crate::operation::start_dashboard_snapshot_job_schedule::builders::StartDashboardSnapshotJobScheduleInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobScheduleOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobScheduleError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.start_dashboard_snapshot_job_schedule();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `StartDashboardSnapshotJobSchedule`.
///
/// <p>Starts an asynchronous job that runs an existing dashboard schedule and sends the dashboard snapshot through email.</p>
/// <p>Only one job can run simultaneously in a given schedule. Repeated requests are skipped with a <code>202</code> HTTP status code.</p>
/// <p>For more information, see <a href="https://docs.aws.amazon.com/quicksight/latest/user/sending-reports.html">Scheduling and sending Amazon QuickSight reports by email</a> and <a href="https://docs.aws.amazon.com/quicksight/latest/user/email-reports-from-dashboard.html">Configuring email report settings for a Amazon QuickSight dashboard</a> in the <i>Amazon QuickSight User Guide</i>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct StartDashboardSnapshotJobScheduleFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::start_dashboard_snapshot_job_schedule::builders::StartDashboardSnapshotJobScheduleInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobScheduleOutput,
        crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobScheduleError,
    > for StartDashboardSnapshotJobScheduleFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobScheduleOutput,
            crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobScheduleError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl StartDashboardSnapshotJobScheduleFluentBuilder {
    /// Creates a new `StartDashboardSnapshotJobScheduleFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the StartDashboardSnapshotJobSchedule as a reference.
    pub fn as_input(&self) -> &crate::operation::start_dashboard_snapshot_job_schedule::builders::StartDashboardSnapshotJobScheduleInputBuilder {
        &self.inner
    }
    /// Sends the request and returns the response.
    ///
    /// If an error occurs, an `SdkError` will be returned with additional details that
    /// can be matched against.
    ///
    /// By default, any retryable failures will be retried twice. Retry behavior
    /// is configurable with the [RetryConfig](aws_smithy_types::retry::RetryConfig), which can be
    /// set when configuring the client.
    pub async fn send(
        self,
    ) -> ::std::result::Result<
        crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobScheduleOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobScheduleError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobSchedule::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobSchedule::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobScheduleOutput,
        crate::operation::start_dashboard_snapshot_job_schedule::StartDashboardSnapshotJobScheduleError,
        Self,
    > {
        crate::client::customize::CustomizableOperation::new(self)
    }
    pub(crate) fn config_override(mut self, config_override: impl ::std::convert::Into<crate::config::Builder>) -> Self {
        self.set_config_override(::std::option::Option::Some(config_override.into()));
        self
    }

    pub(crate) fn set_config_override(&mut self, config_override: ::std::option::Option<crate::config::Builder>) -> &mut Self {
        self.config_override = config_override;
        self
    }
    /// <p>The ID of the Amazon Web Services account that the dashboard snapshot job is executed in.</p>
    pub fn aws_account_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.aws_account_id(input.into());
        self
    }
    /// <p>The ID of the Amazon Web Services account that the dashboard snapshot job is executed in.</p>
    pub fn set_aws_account_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_aws_account_id(input);
        self
    }
    /// <p>The ID of the Amazon Web Services account that the dashboard snapshot job is executed in.</p>
    pub fn get_aws_account_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_aws_account_id()
    }
    /// <p>The ID of the dashboard that you want to start a snapshot job schedule for.</p>
    pub fn dashboard_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.dashboard_id(input.into());
        self
    }
    /// <p>The ID of the dashboard that you want to start a snapshot job schedule for.</p>
    pub fn set_dashboard_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_dashboard_id(input);
        self
    }
    /// <p>The ID of the dashboard that you want to start a snapshot job schedule for.</p>
    pub fn get_dashboard_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_dashboard_id()
    }
    /// <p>The ID of the schedule that you want to start a snapshot job schedule for. The schedule ID can be found in the Amazon QuickSight console in the <b>Schedules</b> pane of the dashboard that the schedule is configured for.</p>
    pub fn schedule_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.schedule_id(input.into());
        self
    }
    /// <p>The ID of the schedule that you want to start a snapshot job schedule for. The schedule ID can be found in the Amazon QuickSight console in the <b>Schedules</b> pane of the dashboard that the schedule is configured for.</p>
    pub fn set_schedule_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_schedule_id(input);
        self
    }
    /// <p>The ID of the schedule that you want to start a snapshot job schedule for. The schedule ID can be found in the Amazon QuickSight console in the <b>Schedules</b> pane of the dashboard that the schedule is configured for.</p>
    pub fn get_schedule_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_schedule_id()
    }
}
