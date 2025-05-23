// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::get_current_metric_data::_get_current_metric_data_output::GetCurrentMetricDataOutputBuilder;

pub use crate::operation::get_current_metric_data::_get_current_metric_data_input::GetCurrentMetricDataInputBuilder;

impl crate::operation::get_current_metric_data::builders::GetCurrentMetricDataInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::get_current_metric_data::GetCurrentMetricDataOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::get_current_metric_data::GetCurrentMetricDataError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.get_current_metric_data();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `GetCurrentMetricData`.
///
/// <p>Gets the real-time metric data from the specified Amazon Connect instance.</p>
/// <p>For a description of each metric, see <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html">Metrics definitions</a> in the <i>Amazon Connect Administrator Guide</i>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct GetCurrentMetricDataFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::get_current_metric_data::builders::GetCurrentMetricDataInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::get_current_metric_data::GetCurrentMetricDataOutput,
        crate::operation::get_current_metric_data::GetCurrentMetricDataError,
    > for GetCurrentMetricDataFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::get_current_metric_data::GetCurrentMetricDataOutput,
            crate::operation::get_current_metric_data::GetCurrentMetricDataError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl GetCurrentMetricDataFluentBuilder {
    /// Creates a new `GetCurrentMetricDataFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the GetCurrentMetricData as a reference.
    pub fn as_input(&self) -> &crate::operation::get_current_metric_data::builders::GetCurrentMetricDataInputBuilder {
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
        crate::operation::get_current_metric_data::GetCurrentMetricDataOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::get_current_metric_data::GetCurrentMetricDataError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::get_current_metric_data::GetCurrentMetricData::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::get_current_metric_data::GetCurrentMetricData::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::get_current_metric_data::GetCurrentMetricDataOutput,
        crate::operation::get_current_metric_data::GetCurrentMetricDataError,
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
    /// Create a paginator for this request
    ///
    /// Paginators are used by calling [`send().await`](crate::operation::get_current_metric_data::paginator::GetCurrentMetricDataPaginator::send) which returns a [`PaginationStream`](aws_smithy_async::future::pagination_stream::PaginationStream).
    pub fn into_paginator(self) -> crate::operation::get_current_metric_data::paginator::GetCurrentMetricDataPaginator {
        crate::operation::get_current_metric_data::paginator::GetCurrentMetricDataPaginator::new(self.handle, self.inner)
    }
    /// <p>The identifier of the Amazon Connect instance. You can <a href="https://docs.aws.amazon.com/connect/latest/adminguide/find-instance-arn.html">find the instance ID</a> in the Amazon Resource Name (ARN) of the instance.</p>
    pub fn instance_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.instance_id(input.into());
        self
    }
    /// <p>The identifier of the Amazon Connect instance. You can <a href="https://docs.aws.amazon.com/connect/latest/adminguide/find-instance-arn.html">find the instance ID</a> in the Amazon Resource Name (ARN) of the instance.</p>
    pub fn set_instance_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_instance_id(input);
        self
    }
    /// <p>The identifier of the Amazon Connect instance. You can <a href="https://docs.aws.amazon.com/connect/latest/adminguide/find-instance-arn.html">find the instance ID</a> in the Amazon Resource Name (ARN) of the instance.</p>
    pub fn get_instance_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_instance_id()
    }
    /// <p>The filters to apply to returned metrics. You can filter up to the following limits:</p>
    /// <ul>
    /// <li>
    /// <p>Queues: 100</p></li>
    /// <li>
    /// <p>Routing profiles: 100</p></li>
    /// <li>
    /// <p>Channels: 3 (VOICE, CHAT, and TASK channels are supported.)</p></li>
    /// <li>
    /// <p>RoutingStepExpressions: 50</p></li>
    /// </ul>
    /// <p>Metric data is retrieved only for the resources associated with the queues or routing profiles, and by any channels included in the filter. (You cannot filter by both queue AND routing profile.) You can include both resource IDs and resource ARNs in the same request.</p>
    /// <p>When using the <code>RoutingStepExpression</code> filter, you need to pass exactly one <code>QueueId</code>. The filter is also case sensitive so when using the <code>RoutingStepExpression</code> filter, grouping by <code>ROUTING_STEP_EXPRESSION</code> is required.</p>
    /// <p>Currently tagging is only supported on the resources that are passed in the filter.</p>
    pub fn filters(mut self, input: crate::types::Filters) -> Self {
        self.inner = self.inner.filters(input);
        self
    }
    /// <p>The filters to apply to returned metrics. You can filter up to the following limits:</p>
    /// <ul>
    /// <li>
    /// <p>Queues: 100</p></li>
    /// <li>
    /// <p>Routing profiles: 100</p></li>
    /// <li>
    /// <p>Channels: 3 (VOICE, CHAT, and TASK channels are supported.)</p></li>
    /// <li>
    /// <p>RoutingStepExpressions: 50</p></li>
    /// </ul>
    /// <p>Metric data is retrieved only for the resources associated with the queues or routing profiles, and by any channels included in the filter. (You cannot filter by both queue AND routing profile.) You can include both resource IDs and resource ARNs in the same request.</p>
    /// <p>When using the <code>RoutingStepExpression</code> filter, you need to pass exactly one <code>QueueId</code>. The filter is also case sensitive so when using the <code>RoutingStepExpression</code> filter, grouping by <code>ROUTING_STEP_EXPRESSION</code> is required.</p>
    /// <p>Currently tagging is only supported on the resources that are passed in the filter.</p>
    pub fn set_filters(mut self, input: ::std::option::Option<crate::types::Filters>) -> Self {
        self.inner = self.inner.set_filters(input);
        self
    }
    /// <p>The filters to apply to returned metrics. You can filter up to the following limits:</p>
    /// <ul>
    /// <li>
    /// <p>Queues: 100</p></li>
    /// <li>
    /// <p>Routing profiles: 100</p></li>
    /// <li>
    /// <p>Channels: 3 (VOICE, CHAT, and TASK channels are supported.)</p></li>
    /// <li>
    /// <p>RoutingStepExpressions: 50</p></li>
    /// </ul>
    /// <p>Metric data is retrieved only for the resources associated with the queues or routing profiles, and by any channels included in the filter. (You cannot filter by both queue AND routing profile.) You can include both resource IDs and resource ARNs in the same request.</p>
    /// <p>When using the <code>RoutingStepExpression</code> filter, you need to pass exactly one <code>QueueId</code>. The filter is also case sensitive so when using the <code>RoutingStepExpression</code> filter, grouping by <code>ROUTING_STEP_EXPRESSION</code> is required.</p>
    /// <p>Currently tagging is only supported on the resources that are passed in the filter.</p>
    pub fn get_filters(&self) -> &::std::option::Option<crate::types::Filters> {
        self.inner.get_filters()
    }
    ///
    /// Appends an item to `Groupings`.
    ///
    /// To override the contents of this collection use [`set_groupings`](Self::set_groupings).
    ///
    /// <p>The grouping applied to the metrics returned. For example, when grouped by <code>QUEUE</code>, the metrics returned apply to each queue rather than aggregated for all queues.</p>
    /// <ul>
    /// <li>
    /// <p>If you group by <code>CHANNEL</code>, you should include a Channels filter. VOICE, CHAT, and TASK channels are supported.</p></li>
    /// <li>
    /// <p>If you group by <code>ROUTING_PROFILE</code>, you must include either a queue or routing profile filter. In addition, a routing profile filter is required for metrics <code>CONTACTS_SCHEDULED</code>, <code>CONTACTS_IN_QUEUE</code>, and <code> OLDEST_CONTACT_AGE</code>.</p></li>
    /// <li>
    /// <p>If no <code>Grouping</code> is included in the request, a summary of metrics is returned.</p></li>
    /// <li>
    /// <p>When using the <code>RoutingStepExpression</code> filter, group by <code>ROUTING_STEP_EXPRESSION</code> is required.</p></li>
    /// </ul>
    pub fn groupings(mut self, input: crate::types::Grouping) -> Self {
        self.inner = self.inner.groupings(input);
        self
    }
    /// <p>The grouping applied to the metrics returned. For example, when grouped by <code>QUEUE</code>, the metrics returned apply to each queue rather than aggregated for all queues.</p>
    /// <ul>
    /// <li>
    /// <p>If you group by <code>CHANNEL</code>, you should include a Channels filter. VOICE, CHAT, and TASK channels are supported.</p></li>
    /// <li>
    /// <p>If you group by <code>ROUTING_PROFILE</code>, you must include either a queue or routing profile filter. In addition, a routing profile filter is required for metrics <code>CONTACTS_SCHEDULED</code>, <code>CONTACTS_IN_QUEUE</code>, and <code> OLDEST_CONTACT_AGE</code>.</p></li>
    /// <li>
    /// <p>If no <code>Grouping</code> is included in the request, a summary of metrics is returned.</p></li>
    /// <li>
    /// <p>When using the <code>RoutingStepExpression</code> filter, group by <code>ROUTING_STEP_EXPRESSION</code> is required.</p></li>
    /// </ul>
    pub fn set_groupings(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Grouping>>) -> Self {
        self.inner = self.inner.set_groupings(input);
        self
    }
    /// <p>The grouping applied to the metrics returned. For example, when grouped by <code>QUEUE</code>, the metrics returned apply to each queue rather than aggregated for all queues.</p>
    /// <ul>
    /// <li>
    /// <p>If you group by <code>CHANNEL</code>, you should include a Channels filter. VOICE, CHAT, and TASK channels are supported.</p></li>
    /// <li>
    /// <p>If you group by <code>ROUTING_PROFILE</code>, you must include either a queue or routing profile filter. In addition, a routing profile filter is required for metrics <code>CONTACTS_SCHEDULED</code>, <code>CONTACTS_IN_QUEUE</code>, and <code> OLDEST_CONTACT_AGE</code>.</p></li>
    /// <li>
    /// <p>If no <code>Grouping</code> is included in the request, a summary of metrics is returned.</p></li>
    /// <li>
    /// <p>When using the <code>RoutingStepExpression</code> filter, group by <code>ROUTING_STEP_EXPRESSION</code> is required.</p></li>
    /// </ul>
    pub fn get_groupings(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Grouping>> {
        self.inner.get_groupings()
    }
    ///
    /// Appends an item to `CurrentMetrics`.
    ///
    /// To override the contents of this collection use [`set_current_metrics`](Self::set_current_metrics).
    ///
    /// <p>The metrics to retrieve. Specify the name and unit for each metric. The following metrics are available. For a description of all the metrics, see <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html">Metrics definitions</a> in the <i>Amazon Connect Administrator Guide</i>.</p>
    /// <dl>
    /// <dt>
    /// AGENTS_AFTER_CONTACT_WORK
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#aftercallwork-real-time">ACW</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_AVAILABLE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#available-real-time">Available</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ERROR
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#error-real-time">Error</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_NON_PRODUCTIVE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#non-productive-time-real-time">NPT (Non-Productive Time)</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ON_CALL
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#on-call-real-time">On contact</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ON_CONTACT
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#on-call-real-time">On contact</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ONLINE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#online-real-time">Online</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_STAFFED
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#staffed-real-time">Staffed</a></p>
    /// </dd>
    /// <dt>
    /// CONTACTS_IN_QUEUE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#in-queue-real-time">In queue</a></p>
    /// </dd>
    /// <dt>
    /// CONTACTS_SCHEDULED
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#scheduled-real-time">Scheduled</a></p>
    /// </dd>
    /// <dt>
    /// OLDEST_CONTACT_AGE
    /// </dt>
    /// <dd>
    /// <p>Unit: SECONDS</p>
    /// <p>When you use groupings, Unit says SECONDS and the Value is returned in SECONDS.</p>
    /// <p>When you do not use groupings, Unit says SECONDS but the Value is returned in MILLISECONDS. For example, if you get a response like this:</p>
    /// <p><code>{ "Metric": { "Name": "OLDEST_CONTACT_AGE", "Unit": "SECONDS" }, "Value": 24113.0 </code>}</p>
    /// <p>The actual OLDEST_CONTACT_AGE is 24 seconds.</p>
    /// <p>When the filter <code>RoutingStepExpression</code> is used, this metric is still calculated from enqueue time. For example, if a contact that has been queued under <code><expression 1></expression></code> for 10 seconds has expired and <code><expression 2></expression></code> becomes active, then <code>OLDEST_CONTACT_AGE</code> for this queue will be counted starting from 10, not 0.</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#oldest-real-time">Oldest</a></p>
    /// </dd>
    /// <dt>
    /// SLOTS_ACTIVE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#active-real-time">Active</a></p>
    /// </dd>
    /// <dt>
    /// SLOTS_AVAILABLE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#availability-real-time">Availability</a></p>
    /// </dd>
    /// </dl>
    pub fn current_metrics(mut self, input: crate::types::CurrentMetric) -> Self {
        self.inner = self.inner.current_metrics(input);
        self
    }
    /// <p>The metrics to retrieve. Specify the name and unit for each metric. The following metrics are available. For a description of all the metrics, see <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html">Metrics definitions</a> in the <i>Amazon Connect Administrator Guide</i>.</p>
    /// <dl>
    /// <dt>
    /// AGENTS_AFTER_CONTACT_WORK
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#aftercallwork-real-time">ACW</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_AVAILABLE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#available-real-time">Available</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ERROR
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#error-real-time">Error</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_NON_PRODUCTIVE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#non-productive-time-real-time">NPT (Non-Productive Time)</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ON_CALL
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#on-call-real-time">On contact</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ON_CONTACT
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#on-call-real-time">On contact</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ONLINE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#online-real-time">Online</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_STAFFED
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#staffed-real-time">Staffed</a></p>
    /// </dd>
    /// <dt>
    /// CONTACTS_IN_QUEUE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#in-queue-real-time">In queue</a></p>
    /// </dd>
    /// <dt>
    /// CONTACTS_SCHEDULED
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#scheduled-real-time">Scheduled</a></p>
    /// </dd>
    /// <dt>
    /// OLDEST_CONTACT_AGE
    /// </dt>
    /// <dd>
    /// <p>Unit: SECONDS</p>
    /// <p>When you use groupings, Unit says SECONDS and the Value is returned in SECONDS.</p>
    /// <p>When you do not use groupings, Unit says SECONDS but the Value is returned in MILLISECONDS. For example, if you get a response like this:</p>
    /// <p><code>{ "Metric": { "Name": "OLDEST_CONTACT_AGE", "Unit": "SECONDS" }, "Value": 24113.0 </code>}</p>
    /// <p>The actual OLDEST_CONTACT_AGE is 24 seconds.</p>
    /// <p>When the filter <code>RoutingStepExpression</code> is used, this metric is still calculated from enqueue time. For example, if a contact that has been queued under <code><expression 1></expression></code> for 10 seconds has expired and <code><expression 2></expression></code> becomes active, then <code>OLDEST_CONTACT_AGE</code> for this queue will be counted starting from 10, not 0.</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#oldest-real-time">Oldest</a></p>
    /// </dd>
    /// <dt>
    /// SLOTS_ACTIVE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#active-real-time">Active</a></p>
    /// </dd>
    /// <dt>
    /// SLOTS_AVAILABLE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#availability-real-time">Availability</a></p>
    /// </dd>
    /// </dl>
    pub fn set_current_metrics(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::CurrentMetric>>) -> Self {
        self.inner = self.inner.set_current_metrics(input);
        self
    }
    /// <p>The metrics to retrieve. Specify the name and unit for each metric. The following metrics are available. For a description of all the metrics, see <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html">Metrics definitions</a> in the <i>Amazon Connect Administrator Guide</i>.</p>
    /// <dl>
    /// <dt>
    /// AGENTS_AFTER_CONTACT_WORK
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#aftercallwork-real-time">ACW</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_AVAILABLE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#available-real-time">Available</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ERROR
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#error-real-time">Error</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_NON_PRODUCTIVE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#non-productive-time-real-time">NPT (Non-Productive Time)</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ON_CALL
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#on-call-real-time">On contact</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ON_CONTACT
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#on-call-real-time">On contact</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_ONLINE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#online-real-time">Online</a></p>
    /// </dd>
    /// <dt>
    /// AGENTS_STAFFED
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#staffed-real-time">Staffed</a></p>
    /// </dd>
    /// <dt>
    /// CONTACTS_IN_QUEUE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#in-queue-real-time">In queue</a></p>
    /// </dd>
    /// <dt>
    /// CONTACTS_SCHEDULED
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#scheduled-real-time">Scheduled</a></p>
    /// </dd>
    /// <dt>
    /// OLDEST_CONTACT_AGE
    /// </dt>
    /// <dd>
    /// <p>Unit: SECONDS</p>
    /// <p>When you use groupings, Unit says SECONDS and the Value is returned in SECONDS.</p>
    /// <p>When you do not use groupings, Unit says SECONDS but the Value is returned in MILLISECONDS. For example, if you get a response like this:</p>
    /// <p><code>{ "Metric": { "Name": "OLDEST_CONTACT_AGE", "Unit": "SECONDS" }, "Value": 24113.0 </code>}</p>
    /// <p>The actual OLDEST_CONTACT_AGE is 24 seconds.</p>
    /// <p>When the filter <code>RoutingStepExpression</code> is used, this metric is still calculated from enqueue time. For example, if a contact that has been queued under <code><expression 1></expression></code> for 10 seconds has expired and <code><expression 2></expression></code> becomes active, then <code>OLDEST_CONTACT_AGE</code> for this queue will be counted starting from 10, not 0.</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#oldest-real-time">Oldest</a></p>
    /// </dd>
    /// <dt>
    /// SLOTS_ACTIVE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#active-real-time">Active</a></p>
    /// </dd>
    /// <dt>
    /// SLOTS_AVAILABLE
    /// </dt>
    /// <dd>
    /// <p>Unit: COUNT</p>
    /// <p>Name in real-time metrics report: <a href="https://docs.aws.amazon.com/connect/latest/adminguide/metrics-definitions.html#availability-real-time">Availability</a></p>
    /// </dd>
    /// </dl>
    pub fn get_current_metrics(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::CurrentMetric>> {
        self.inner.get_current_metrics()
    }
    /// <p>The token for the next set of results. Use the value returned in the previous response in the next request to retrieve the next set of results.</p>
    /// <p>The token expires after 5 minutes from the time it is created. Subsequent requests that use the token must use the same request parameters as the request that generated the token.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.next_token(input.into());
        self
    }
    /// <p>The token for the next set of results. Use the value returned in the previous response in the next request to retrieve the next set of results.</p>
    /// <p>The token expires after 5 minutes from the time it is created. Subsequent requests that use the token must use the same request parameters as the request that generated the token.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_next_token(input);
        self
    }
    /// <p>The token for the next set of results. Use the value returned in the previous response in the next request to retrieve the next set of results.</p>
    /// <p>The token expires after 5 minutes from the time it is created. Subsequent requests that use the token must use the same request parameters as the request that generated the token.</p>
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_next_token()
    }
    /// <p>The maximum number of results to return per page.</p>
    pub fn max_results(mut self, input: i32) -> Self {
        self.inner = self.inner.max_results(input);
        self
    }
    /// <p>The maximum number of results to return per page.</p>
    pub fn set_max_results(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_max_results(input);
        self
    }
    /// <p>The maximum number of results to return per page.</p>
    pub fn get_max_results(&self) -> &::std::option::Option<i32> {
        self.inner.get_max_results()
    }
    ///
    /// Appends an item to `SortCriteria`.
    ///
    /// To override the contents of this collection use [`set_sort_criteria`](Self::set_sort_criteria).
    ///
    /// <p>The way to sort the resulting response based on metrics. You can enter one sort criteria. By default resources are sorted based on <code>AGENTS_ONLINE</code>, <code>DESCENDING</code>. The metric collection is sorted based on the input metrics.</p>
    /// <p>Note the following:</p>
    /// <ul>
    /// <li>
    /// <p>Sorting on <code>SLOTS_ACTIVE</code> and <code>SLOTS_AVAILABLE</code> is not supported.</p></li>
    /// </ul>
    pub fn sort_criteria(mut self, input: crate::types::CurrentMetricSortCriteria) -> Self {
        self.inner = self.inner.sort_criteria(input);
        self
    }
    /// <p>The way to sort the resulting response based on metrics. You can enter one sort criteria. By default resources are sorted based on <code>AGENTS_ONLINE</code>, <code>DESCENDING</code>. The metric collection is sorted based on the input metrics.</p>
    /// <p>Note the following:</p>
    /// <ul>
    /// <li>
    /// <p>Sorting on <code>SLOTS_ACTIVE</code> and <code>SLOTS_AVAILABLE</code> is not supported.</p></li>
    /// </ul>
    pub fn set_sort_criteria(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::CurrentMetricSortCriteria>>) -> Self {
        self.inner = self.inner.set_sort_criteria(input);
        self
    }
    /// <p>The way to sort the resulting response based on metrics. You can enter one sort criteria. By default resources are sorted based on <code>AGENTS_ONLINE</code>, <code>DESCENDING</code>. The metric collection is sorted based on the input metrics.</p>
    /// <p>Note the following:</p>
    /// <ul>
    /// <li>
    /// <p>Sorting on <code>SLOTS_ACTIVE</code> and <code>SLOTS_AVAILABLE</code> is not supported.</p></li>
    /// </ul>
    pub fn get_sort_criteria(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::CurrentMetricSortCriteria>> {
        self.inner.get_sort_criteria()
    }
}
