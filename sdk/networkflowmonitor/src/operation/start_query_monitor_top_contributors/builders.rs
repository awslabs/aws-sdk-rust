// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::start_query_monitor_top_contributors::_start_query_monitor_top_contributors_output::StartQueryMonitorTopContributorsOutputBuilder;

pub use crate::operation::start_query_monitor_top_contributors::_start_query_monitor_top_contributors_input::StartQueryMonitorTopContributorsInputBuilder;

impl crate::operation::start_query_monitor_top_contributors::builders::StartQueryMonitorTopContributorsInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributorsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributorsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.start_query_monitor_top_contributors();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `StartQueryMonitorTopContributors`.
///
/// <p>Create a query that you can use with the Network Flow Monitor query interface to return the top contributors for a monitor. Specify the monitor that you want to create the query for.</p>
/// <p>The call returns a query ID that you can use with <a href="https://docs.aws.amazon.com/networkflowmonitor/2.0/APIReference/API_GetQueryResultsMonitorTopContributors.html"> GetQueryResultsMonitorTopContributors</a> to run the query and return the top contributors for a specific monitor.</p>
/// <p>Top contributors in Network Flow Monitor are network flows with the highest values for a specific metric type. Top contributors can be across all workload insights, for a given scope, or for a specific monitor. Use the applicable APIs for the top contributors that you want to be returned.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct StartQueryMonitorTopContributorsFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::start_query_monitor_top_contributors::builders::StartQueryMonitorTopContributorsInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributorsOutput,
        crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributorsError,
    > for StartQueryMonitorTopContributorsFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributorsOutput,
            crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributorsError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl StartQueryMonitorTopContributorsFluentBuilder {
    /// Creates a new `StartQueryMonitorTopContributorsFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the StartQueryMonitorTopContributors as a reference.
    pub fn as_input(&self) -> &crate::operation::start_query_monitor_top_contributors::builders::StartQueryMonitorTopContributorsInputBuilder {
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
        crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributorsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributorsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributors::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributors::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributorsOutput,
        crate::operation::start_query_monitor_top_contributors::StartQueryMonitorTopContributorsError,
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
    /// <p>The name of the monitor.</p>
    pub fn monitor_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.monitor_name(input.into());
        self
    }
    /// <p>The name of the monitor.</p>
    pub fn set_monitor_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_monitor_name(input);
        self
    }
    /// <p>The name of the monitor.</p>
    pub fn get_monitor_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_monitor_name()
    }
    /// <p>The timestamp that is the date and time beginning of the period that you want to retrieve results for with your query.</p>
    pub fn start_time(mut self, input: ::aws_smithy_types::DateTime) -> Self {
        self.inner = self.inner.start_time(input);
        self
    }
    /// <p>The timestamp that is the date and time beginning of the period that you want to retrieve results for with your query.</p>
    pub fn set_start_time(mut self, input: ::std::option::Option<::aws_smithy_types::DateTime>) -> Self {
        self.inner = self.inner.set_start_time(input);
        self
    }
    /// <p>The timestamp that is the date and time beginning of the period that you want to retrieve results for with your query.</p>
    pub fn get_start_time(&self) -> &::std::option::Option<::aws_smithy_types::DateTime> {
        self.inner.get_start_time()
    }
    /// <p>The timestamp that is the date and time end of the period that you want to retrieve results for with your query.</p>
    pub fn end_time(mut self, input: ::aws_smithy_types::DateTime) -> Self {
        self.inner = self.inner.end_time(input);
        self
    }
    /// <p>The timestamp that is the date and time end of the period that you want to retrieve results for with your query.</p>
    pub fn set_end_time(mut self, input: ::std::option::Option<::aws_smithy_types::DateTime>) -> Self {
        self.inner = self.inner.set_end_time(input);
        self
    }
    /// <p>The timestamp that is the date and time end of the period that you want to retrieve results for with your query.</p>
    pub fn get_end_time(&self) -> &::std::option::Option<::aws_smithy_types::DateTime> {
        self.inner.get_end_time()
    }
    /// <p>The metric that you want to query top contributors for. That is, you can specify a metric with this call and return the top contributor network flows, for that type of metric, for a monitor and (optionally) within a specific category, such as network flows between Availability Zones.</p>
    pub fn metric_name(mut self, input: crate::types::MonitorMetric) -> Self {
        self.inner = self.inner.metric_name(input);
        self
    }
    /// <p>The metric that you want to query top contributors for. That is, you can specify a metric with this call and return the top contributor network flows, for that type of metric, for a monitor and (optionally) within a specific category, such as network flows between Availability Zones.</p>
    pub fn set_metric_name(mut self, input: ::std::option::Option<crate::types::MonitorMetric>) -> Self {
        self.inner = self.inner.set_metric_name(input);
        self
    }
    /// <p>The metric that you want to query top contributors for. That is, you can specify a metric with this call and return the top contributor network flows, for that type of metric, for a monitor and (optionally) within a specific category, such as network flows between Availability Zones.</p>
    pub fn get_metric_name(&self) -> &::std::option::Option<crate::types::MonitorMetric> {
        self.inner.get_metric_name()
    }
    /// <p>The category that you want to query top contributors for, for a specific monitor. Destination categories can be one of the following:</p>
    /// <ul>
    /// <li>
    /// <p><code>INTRA_AZ</code>: Top contributor network flows within a single Availability Zone</p></li>
    /// <li>
    /// <p><code>INTER_AZ</code>: Top contributor network flows between Availability Zones</p></li>
    /// <li>
    /// <p><code>INTER_VPC</code>: Top contributor network flows between VPCs</p></li>
    /// <li>
    /// <p><code>AMAZON_S3</code>: Top contributor network flows to or from Amazon S3</p></li>
    /// <li>
    /// <p><code>AMAZON_DYNAMODB</code>: Top contributor network flows to or from Amazon Dynamo DB</p></li>
    /// <li>
    /// <p><code>UNCLASSIFIED</code>: Top contributor network flows that do not have a bucket classification</p></li>
    /// </ul>
    pub fn destination_category(mut self, input: crate::types::DestinationCategory) -> Self {
        self.inner = self.inner.destination_category(input);
        self
    }
    /// <p>The category that you want to query top contributors for, for a specific monitor. Destination categories can be one of the following:</p>
    /// <ul>
    /// <li>
    /// <p><code>INTRA_AZ</code>: Top contributor network flows within a single Availability Zone</p></li>
    /// <li>
    /// <p><code>INTER_AZ</code>: Top contributor network flows between Availability Zones</p></li>
    /// <li>
    /// <p><code>INTER_VPC</code>: Top contributor network flows between VPCs</p></li>
    /// <li>
    /// <p><code>AMAZON_S3</code>: Top contributor network flows to or from Amazon S3</p></li>
    /// <li>
    /// <p><code>AMAZON_DYNAMODB</code>: Top contributor network flows to or from Amazon Dynamo DB</p></li>
    /// <li>
    /// <p><code>UNCLASSIFIED</code>: Top contributor network flows that do not have a bucket classification</p></li>
    /// </ul>
    pub fn set_destination_category(mut self, input: ::std::option::Option<crate::types::DestinationCategory>) -> Self {
        self.inner = self.inner.set_destination_category(input);
        self
    }
    /// <p>The category that you want to query top contributors for, for a specific monitor. Destination categories can be one of the following:</p>
    /// <ul>
    /// <li>
    /// <p><code>INTRA_AZ</code>: Top contributor network flows within a single Availability Zone</p></li>
    /// <li>
    /// <p><code>INTER_AZ</code>: Top contributor network flows between Availability Zones</p></li>
    /// <li>
    /// <p><code>INTER_VPC</code>: Top contributor network flows between VPCs</p></li>
    /// <li>
    /// <p><code>AMAZON_S3</code>: Top contributor network flows to or from Amazon S3</p></li>
    /// <li>
    /// <p><code>AMAZON_DYNAMODB</code>: Top contributor network flows to or from Amazon Dynamo DB</p></li>
    /// <li>
    /// <p><code>UNCLASSIFIED</code>: Top contributor network flows that do not have a bucket classification</p></li>
    /// </ul>
    pub fn get_destination_category(&self) -> &::std::option::Option<crate::types::DestinationCategory> {
        self.inner.get_destination_category()
    }
    /// <p>The maximum number of top contributors to return.</p>
    pub fn limit(mut self, input: i32) -> Self {
        self.inner = self.inner.limit(input);
        self
    }
    /// <p>The maximum number of top contributors to return.</p>
    pub fn set_limit(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_limit(input);
        self
    }
    /// <p>The maximum number of top contributors to return.</p>
    pub fn get_limit(&self) -> &::std::option::Option<i32> {
        self.inner.get_limit()
    }
}
