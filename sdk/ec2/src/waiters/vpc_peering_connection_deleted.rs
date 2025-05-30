// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

///
/// Fluent builder for the `vpc_peering_connection_deleted` waiter.
///
/// This builder is intended to be used similar to the other fluent builders for
/// normal operations on the client. However, instead of a `send` method, it has
/// a `wait` method that takes a maximum amount of time to wait.
///
/// Construct this fluent builder using the client by importing the
/// [`Waiters`](crate::client::Waiters) trait and calling the methods
/// prefixed with `wait_until`.
///
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct VpcPeeringConnectionDeletedFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::describe_vpc_peering_connections::builders::DescribeVpcPeeringConnectionsInputBuilder,
}
impl VpcPeeringConnectionDeletedFluentBuilder {
    /// Creates a new `VpcPeeringConnectionDeletedFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
        }
    }
    /// Access the DescribeVpcPeeringConnections as a reference.
    pub fn as_input(&self) -> &crate::operation::describe_vpc_peering_connections::builders::DescribeVpcPeeringConnectionsInputBuilder {
        &self.inner
    }
    /// Wait for `vpc_peering_connection_deleted`
    pub async fn wait(
        self,
        max_wait: ::std::time::Duration,
    ) -> ::std::result::Result<
        crate::waiters::vpc_peering_connection_deleted::VpcPeeringConnectionDeletedFinalPoll,
        crate::waiters::vpc_peering_connection_deleted::WaitUntilVpcPeeringConnectionDeletedError,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::waiters::error::WaiterError::construction_failure)?;
        let runtime_plugins = crate::operation::describe_vpc_peering_connections::DescribeVpcPeeringConnections::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            ::std::option::Option::None,
        )
        .with_operation_plugin(crate::sdk_feature_tracker::waiter::WaiterFeatureTrackerRuntimePlugin::new());
        let mut cfg = ::aws_smithy_types::config_bag::ConfigBag::base();
        let runtime_components_builder = runtime_plugins
            .apply_client_configuration(&mut cfg)
            .map_err(::aws_smithy_runtime_api::client::waiters::error::WaiterError::construction_failure)?;
        let time_components = runtime_components_builder.into_time_components();
        let sleep_impl = time_components.sleep_impl().expect("a sleep impl is required by waiters");
        let time_source = time_components.time_source().expect("a time source is required by waiters");

        let acceptor = move |result: ::std::result::Result<
            &crate::operation::describe_vpc_peering_connections::DescribeVpcPeeringConnectionsOutput,
            &crate::operation::describe_vpc_peering_connections::DescribeVpcPeeringConnectionsError,
        >| {
            // Matches: {"output":{"path":"VpcPeeringConnections[].Status.Code","expected":"deleted","comparator":"allStringEquals"}}
            if crate::waiters::matchers::match_describe_vpc_peering_connections_38dded2097a8f81f9(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Success;
            }
            // Matches: {"errorType":"InvalidVpcPeeringConnectionID.NotFound"}
            if crate::waiters::matchers::match_describe_vpc_peering_connections_e0cb68a203dc3e8d0(result) {
                return ::aws_smithy_runtime::client::waiters::AcceptorState::Success;
            }
            ::aws_smithy_runtime::client::waiters::AcceptorState::NoAcceptorsMatched
        };
        let operation = move || {
            let input = input.clone();
            let runtime_plugins = runtime_plugins.clone();
            async move { crate::operation::describe_vpc_peering_connections::DescribeVpcPeeringConnections::orchestrate(&runtime_plugins, input).await }
        };
        let orchestrator = ::aws_smithy_runtime::client::waiters::WaiterOrchestrator::builder()
            .min_delay(::std::time::Duration::from_secs(15))
            .max_delay(::std::time::Duration::from_secs(120))
            .max_wait(max_wait)
            .time_source(time_source)
            .sleep_impl(sleep_impl)
            .acceptor(acceptor)
            .operation(operation)
            .build();
        ::aws_smithy_runtime::client::waiters::attach_waiter_tracing_span(orchestrator.orchestrate()).await
    }
    /// <p>The token returned from a previous paginated request. Pagination continues from the end of the items returned by the previous request.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.next_token(input.into());
        self
    }
    /// <p>The token returned from a previous paginated request. Pagination continues from the end of the items returned by the previous request.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_next_token(input);
        self
    }
    /// <p>The token returned from a previous paginated request. Pagination continues from the end of the items returned by the previous request.</p>
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_next_token()
    }
    /// <p>The maximum number of items to return for this request. To get the next page of items, make another request with the token returned in the output. For more information, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Query-Requests.html#api-pagination">Pagination</a>.</p>
    pub fn max_results(mut self, input: i32) -> Self {
        self.inner = self.inner.max_results(input);
        self
    }
    /// <p>The maximum number of items to return for this request. To get the next page of items, make another request with the token returned in the output. For more information, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Query-Requests.html#api-pagination">Pagination</a>.</p>
    pub fn set_max_results(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_max_results(input);
        self
    }
    /// <p>The maximum number of items to return for this request. To get the next page of items, make another request with the token returned in the output. For more information, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Query-Requests.html#api-pagination">Pagination</a>.</p>
    pub fn get_max_results(&self) -> &::std::option::Option<i32> {
        self.inner.get_max_results()
    }
    /// <p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p>
    pub fn dry_run(mut self, input: bool) -> Self {
        self.inner = self.inner.dry_run(input);
        self
    }
    /// <p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p>
    pub fn set_dry_run(mut self, input: ::std::option::Option<bool>) -> Self {
        self.inner = self.inner.set_dry_run(input);
        self
    }
    /// <p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p>
    pub fn get_dry_run(&self) -> &::std::option::Option<bool> {
        self.inner.get_dry_run()
    }
    ///
    /// Appends an item to `VpcPeeringConnectionIds`.
    ///
    /// To override the contents of this collection use [`set_vpc_peering_connection_ids`](Self::set_vpc_peering_connection_ids).
    ///
    /// <p>The IDs of the VPC peering connections.</p>
    /// <p>Default: Describes all your VPC peering connections.</p>
    pub fn vpc_peering_connection_ids(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.vpc_peering_connection_ids(input.into());
        self
    }
    /// <p>The IDs of the VPC peering connections.</p>
    /// <p>Default: Describes all your VPC peering connections.</p>
    pub fn set_vpc_peering_connection_ids(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_vpc_peering_connection_ids(input);
        self
    }
    /// <p>The IDs of the VPC peering connections.</p>
    /// <p>Default: Describes all your VPC peering connections.</p>
    pub fn get_vpc_peering_connection_ids(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_vpc_peering_connection_ids()
    }
    ///
    /// Appends an item to `Filters`.
    ///
    /// To override the contents of this collection use [`set_filters`](Self::set_filters).
    ///
    /// <p>The filters.</p>
    /// <ul>
    /// <li>
    /// <p><code>accepter-vpc-info.cidr-block</code> - The IPv4 CIDR block of the accepter VPC.</p></li>
    /// <li>
    /// <p><code>accepter-vpc-info.owner-id</code> - The ID of the Amazon Web Services account that owns the accepter VPC.</p></li>
    /// <li>
    /// <p><code>accepter-vpc-info.vpc-id</code> - The ID of the accepter VPC.</p></li>
    /// <li>
    /// <p><code>expiration-time</code> - The expiration date and time for the VPC peering connection.</p></li>
    /// <li>
    /// <p><code>requester-vpc-info.cidr-block</code> - The IPv4 CIDR block of the requester's VPC.</p></li>
    /// <li>
    /// <p><code>requester-vpc-info.owner-id</code> - The ID of the Amazon Web Services account that owns the requester VPC.</p></li>
    /// <li>
    /// <p><code>requester-vpc-info.vpc-id</code> - The ID of the requester VPC.</p></li>
    /// <li>
    /// <p><code>status-code</code> - The status of the VPC peering connection (<code>pending-acceptance</code> | <code>failed</code> | <code>expired</code> | <code>provisioning</code> | <code>active</code> | <code>deleting</code> | <code>deleted</code> | <code>rejected</code>).</p></li>
    /// <li>
    /// <p><code>status-message</code> - A message that provides more information about the status of the VPC peering connection, if applicable.</p></li>
    /// <li>
    /// <p><code>tag</code> - The key/value combination of a tag assigned to the resource. Use the tag key in the filter name and the tag value as the filter value. For example, to find all resources that have a tag with the key <code>Owner</code> and the value <code>TeamA</code>, specify <code>tag:Owner</code> for the filter name and <code>TeamA</code> for the filter value.</p></li>
    /// <li>
    /// <p><code>tag-key</code> - The key of a tag assigned to the resource. Use this filter to find all resources assigned a tag with a specific key, regardless of the tag value.</p></li>
    /// <li>
    /// <p><code>vpc-peering-connection-id</code> - The ID of the VPC peering connection.</p></li>
    /// </ul>
    pub fn filters(mut self, input: crate::types::Filter) -> Self {
        self.inner = self.inner.filters(input);
        self
    }
    /// <p>The filters.</p>
    /// <ul>
    /// <li>
    /// <p><code>accepter-vpc-info.cidr-block</code> - The IPv4 CIDR block of the accepter VPC.</p></li>
    /// <li>
    /// <p><code>accepter-vpc-info.owner-id</code> - The ID of the Amazon Web Services account that owns the accepter VPC.</p></li>
    /// <li>
    /// <p><code>accepter-vpc-info.vpc-id</code> - The ID of the accepter VPC.</p></li>
    /// <li>
    /// <p><code>expiration-time</code> - The expiration date and time for the VPC peering connection.</p></li>
    /// <li>
    /// <p><code>requester-vpc-info.cidr-block</code> - The IPv4 CIDR block of the requester's VPC.</p></li>
    /// <li>
    /// <p><code>requester-vpc-info.owner-id</code> - The ID of the Amazon Web Services account that owns the requester VPC.</p></li>
    /// <li>
    /// <p><code>requester-vpc-info.vpc-id</code> - The ID of the requester VPC.</p></li>
    /// <li>
    /// <p><code>status-code</code> - The status of the VPC peering connection (<code>pending-acceptance</code> | <code>failed</code> | <code>expired</code> | <code>provisioning</code> | <code>active</code> | <code>deleting</code> | <code>deleted</code> | <code>rejected</code>).</p></li>
    /// <li>
    /// <p><code>status-message</code> - A message that provides more information about the status of the VPC peering connection, if applicable.</p></li>
    /// <li>
    /// <p><code>tag</code> - The key/value combination of a tag assigned to the resource. Use the tag key in the filter name and the tag value as the filter value. For example, to find all resources that have a tag with the key <code>Owner</code> and the value <code>TeamA</code>, specify <code>tag:Owner</code> for the filter name and <code>TeamA</code> for the filter value.</p></li>
    /// <li>
    /// <p><code>tag-key</code> - The key of a tag assigned to the resource. Use this filter to find all resources assigned a tag with a specific key, regardless of the tag value.</p></li>
    /// <li>
    /// <p><code>vpc-peering-connection-id</code> - The ID of the VPC peering connection.</p></li>
    /// </ul>
    pub fn set_filters(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Filter>>) -> Self {
        self.inner = self.inner.set_filters(input);
        self
    }
    /// <p>The filters.</p>
    /// <ul>
    /// <li>
    /// <p><code>accepter-vpc-info.cidr-block</code> - The IPv4 CIDR block of the accepter VPC.</p></li>
    /// <li>
    /// <p><code>accepter-vpc-info.owner-id</code> - The ID of the Amazon Web Services account that owns the accepter VPC.</p></li>
    /// <li>
    /// <p><code>accepter-vpc-info.vpc-id</code> - The ID of the accepter VPC.</p></li>
    /// <li>
    /// <p><code>expiration-time</code> - The expiration date and time for the VPC peering connection.</p></li>
    /// <li>
    /// <p><code>requester-vpc-info.cidr-block</code> - The IPv4 CIDR block of the requester's VPC.</p></li>
    /// <li>
    /// <p><code>requester-vpc-info.owner-id</code> - The ID of the Amazon Web Services account that owns the requester VPC.</p></li>
    /// <li>
    /// <p><code>requester-vpc-info.vpc-id</code> - The ID of the requester VPC.</p></li>
    /// <li>
    /// <p><code>status-code</code> - The status of the VPC peering connection (<code>pending-acceptance</code> | <code>failed</code> | <code>expired</code> | <code>provisioning</code> | <code>active</code> | <code>deleting</code> | <code>deleted</code> | <code>rejected</code>).</p></li>
    /// <li>
    /// <p><code>status-message</code> - A message that provides more information about the status of the VPC peering connection, if applicable.</p></li>
    /// <li>
    /// <p><code>tag</code> - The key/value combination of a tag assigned to the resource. Use the tag key in the filter name and the tag value as the filter value. For example, to find all resources that have a tag with the key <code>Owner</code> and the value <code>TeamA</code>, specify <code>tag:Owner</code> for the filter name and <code>TeamA</code> for the filter value.</p></li>
    /// <li>
    /// <p><code>tag-key</code> - The key of a tag assigned to the resource. Use this filter to find all resources assigned a tag with a specific key, regardless of the tag value.</p></li>
    /// <li>
    /// <p><code>vpc-peering-connection-id</code> - The ID of the VPC peering connection.</p></li>
    /// </ul>
    pub fn get_filters(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Filter>> {
        self.inner.get_filters()
    }
}

/// Successful return type for the `vpc_peering_connection_deleted` waiter.
pub type VpcPeeringConnectionDeletedFinalPoll = ::aws_smithy_runtime_api::client::waiters::FinalPoll<
    crate::operation::describe_vpc_peering_connections::DescribeVpcPeeringConnectionsOutput,
    ::aws_smithy_runtime_api::client::result::SdkError<
        crate::operation::describe_vpc_peering_connections::DescribeVpcPeeringConnectionsError,
        ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    >,
>;

/// Error type for the `vpc_peering_connection_deleted` waiter.
pub type WaitUntilVpcPeeringConnectionDeletedError = ::aws_smithy_runtime_api::client::waiters::error::WaiterError<
    crate::operation::describe_vpc_peering_connections::DescribeVpcPeeringConnectionsOutput,
    crate::operation::describe_vpc_peering_connections::DescribeVpcPeeringConnectionsError,
>;
