// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::describe_subnets::_describe_subnets_output::DescribeSubnetsOutputBuilder;

pub use crate::operation::describe_subnets::_describe_subnets_input::DescribeSubnetsInputBuilder;

impl crate::operation::describe_subnets::builders::DescribeSubnetsInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::describe_subnets::DescribeSubnetsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_subnets::DescribeSubnetsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.describe_subnets();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `DescribeSubnets`.
///
/// <p>Describes your subnets. The default is to describe all your subnets. Alternatively, you can specify specific subnet IDs or filter the results to include only the subnets that match specific criteria.</p>
/// <p>For more information, see <a href="https://docs.aws.amazon.com/vpc/latest/userguide/configure-subnets.html">Subnets</a> in the <i>Amazon VPC User Guide</i>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct DescribeSubnetsFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::describe_subnets::builders::DescribeSubnetsInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::describe_subnets::DescribeSubnetsOutput,
        crate::operation::describe_subnets::DescribeSubnetsError,
    > for DescribeSubnetsFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::describe_subnets::DescribeSubnetsOutput,
            crate::operation::describe_subnets::DescribeSubnetsError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl DescribeSubnetsFluentBuilder {
    /// Creates a new `DescribeSubnetsFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the DescribeSubnets as a reference.
    pub fn as_input(&self) -> &crate::operation::describe_subnets::builders::DescribeSubnetsInputBuilder {
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
        crate::operation::describe_subnets::DescribeSubnetsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_subnets::DescribeSubnetsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::describe_subnets::DescribeSubnets::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::describe_subnets::DescribeSubnets::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::describe_subnets::DescribeSubnetsOutput,
        crate::operation::describe_subnets::DescribeSubnetsError,
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
    /// Paginators are used by calling [`send().await`](crate::operation::describe_subnets::paginator::DescribeSubnetsPaginator::send) which returns a [`PaginationStream`](aws_smithy_async::future::pagination_stream::PaginationStream).
    pub fn into_paginator(self) -> crate::operation::describe_subnets::paginator::DescribeSubnetsPaginator {
        crate::operation::describe_subnets::paginator::DescribeSubnetsPaginator::new(self.handle, self.inner)
    }
    ///
    /// Appends an item to `Filters`.
    ///
    /// To override the contents of this collection use [`set_filters`](Self::set_filters).
    ///
    /// <p>The filters.</p>
    /// <ul>
    /// <li>
    /// <p><code>availability-zone</code> - The Availability Zone for the subnet. You can also use <code>availabilityZone</code> as the filter name.</p></li>
    /// <li>
    /// <p><code>availability-zone-id</code> - The ID of the Availability Zone for the subnet. You can also use <code>availabilityZoneId</code> as the filter name.</p></li>
    /// <li>
    /// <p><code>available-ip-address-count</code> - The number of IPv4 addresses in the subnet that are available.</p></li>
    /// <li>
    /// <p><code>cidr-block</code> - The IPv4 CIDR block of the subnet. The CIDR block you specify must exactly match the subnet's CIDR block for information to be returned for the subnet. You can also use <code>cidr</code> or <code>cidrBlock</code> as the filter names.</p></li>
    /// <li>
    /// <p><code>customer-owned-ipv4-pool</code> - The customer-owned IPv4 address pool associated with the subnet.</p></li>
    /// <li>
    /// <p><code>default-for-az</code> - Indicates whether this is the default subnet for the Availability Zone (<code>true</code> | <code>false</code>). You can also use <code>defaultForAz</code> as the filter name.</p></li>
    /// <li>
    /// <p><code>enable-dns64</code> - Indicates whether DNS queries made to the Amazon-provided DNS Resolver in this subnet should return synthetic IPv6 addresses for IPv4-only destinations.</p></li>
    /// <li>
    /// <p><code>enable-lni-at-device-index</code> - Indicates the device position for local network interfaces in this subnet. For example, <code>1</code> indicates local network interfaces in this subnet are the secondary network interface (eth1).</p></li>
    /// <li>
    /// <p><code>ipv6-cidr-block-association.ipv6-cidr-block</code> - An IPv6 CIDR block associated with the subnet.</p></li>
    /// <li>
    /// <p><code>ipv6-cidr-block-association.association-id</code> - An association ID for an IPv6 CIDR block associated with the subnet.</p></li>
    /// <li>
    /// <p><code>ipv6-cidr-block-association.state</code> - The state of an IPv6 CIDR block associated with the subnet.</p></li>
    /// <li>
    /// <p><code>ipv6-native</code> - Indicates whether this is an IPv6 only subnet (<code>true</code> | <code>false</code>).</p></li>
    /// <li>
    /// <p><code>map-customer-owned-ip-on-launch</code> - Indicates whether a network interface created in this subnet (including a network interface created by <code>RunInstances</code>) receives a customer-owned IPv4 address.</p></li>
    /// <li>
    /// <p><code>map-public-ip-on-launch</code> - Indicates whether instances launched in this subnet receive a public IPv4 address.</p></li>
    /// <li>
    /// <p><code>outpost-arn</code> - The Amazon Resource Name (ARN) of the Outpost.</p></li>
    /// <li>
    /// <p><code>owner-id</code> - The ID of the Amazon Web Services account that owns the subnet.</p></li>
    /// <li>
    /// <p><code>private-dns-name-options-on-launch.hostname-type</code> - The type of hostname to assign to instances in the subnet at launch. For IPv4-only and dual-stack (IPv4 and IPv6) subnets, an instance DNS name can be based on the instance IPv4 address (ip-name) or the instance ID (resource-name). For IPv6 only subnets, an instance DNS name must be based on the instance ID (resource-name).</p></li>
    /// <li>
    /// <p><code>private-dns-name-options-on-launch.enable-resource-name-dns-a-record</code> - Indicates whether to respond to DNS queries for instance hostnames with DNS A records.</p></li>
    /// <li>
    /// <p><code>private-dns-name-options-on-launch.enable-resource-name-dns-aaaa-record</code> - Indicates whether to respond to DNS queries for instance hostnames with DNS AAAA records.</p></li>
    /// <li>
    /// <p><code>state</code> - The state of the subnet (<code>pending</code> | <code>available</code>).</p></li>
    /// <li>
    /// <p><code>subnet-arn</code> - The Amazon Resource Name (ARN) of the subnet.</p></li>
    /// <li>
    /// <p><code>subnet-id</code> - The ID of the subnet.</p></li>
    /// <li>
    /// <p><code>tag</code> - The key/value combination of a tag assigned to the resource. Use the tag key in the filter name and the tag value as the filter value. For example, to find all resources that have a tag with the key <code>Owner</code> and the value <code>TeamA</code>, specify <code>tag:Owner</code> for the filter name and <code>TeamA</code> for the filter value.</p></li>
    /// <li>
    /// <p><code>tag-key</code> - The key of a tag assigned to the resource. Use this filter to find all resources assigned a tag with a specific key, regardless of the tag value.</p></li>
    /// <li>
    /// <p><code>vpc-id</code> - The ID of the VPC for the subnet.</p></li>
    /// </ul>
    pub fn filters(mut self, input: crate::types::Filter) -> Self {
        self.inner = self.inner.filters(input);
        self
    }
    /// <p>The filters.</p>
    /// <ul>
    /// <li>
    /// <p><code>availability-zone</code> - The Availability Zone for the subnet. You can also use <code>availabilityZone</code> as the filter name.</p></li>
    /// <li>
    /// <p><code>availability-zone-id</code> - The ID of the Availability Zone for the subnet. You can also use <code>availabilityZoneId</code> as the filter name.</p></li>
    /// <li>
    /// <p><code>available-ip-address-count</code> - The number of IPv4 addresses in the subnet that are available.</p></li>
    /// <li>
    /// <p><code>cidr-block</code> - The IPv4 CIDR block of the subnet. The CIDR block you specify must exactly match the subnet's CIDR block for information to be returned for the subnet. You can also use <code>cidr</code> or <code>cidrBlock</code> as the filter names.</p></li>
    /// <li>
    /// <p><code>customer-owned-ipv4-pool</code> - The customer-owned IPv4 address pool associated with the subnet.</p></li>
    /// <li>
    /// <p><code>default-for-az</code> - Indicates whether this is the default subnet for the Availability Zone (<code>true</code> | <code>false</code>). You can also use <code>defaultForAz</code> as the filter name.</p></li>
    /// <li>
    /// <p><code>enable-dns64</code> - Indicates whether DNS queries made to the Amazon-provided DNS Resolver in this subnet should return synthetic IPv6 addresses for IPv4-only destinations.</p></li>
    /// <li>
    /// <p><code>enable-lni-at-device-index</code> - Indicates the device position for local network interfaces in this subnet. For example, <code>1</code> indicates local network interfaces in this subnet are the secondary network interface (eth1).</p></li>
    /// <li>
    /// <p><code>ipv6-cidr-block-association.ipv6-cidr-block</code> - An IPv6 CIDR block associated with the subnet.</p></li>
    /// <li>
    /// <p><code>ipv6-cidr-block-association.association-id</code> - An association ID for an IPv6 CIDR block associated with the subnet.</p></li>
    /// <li>
    /// <p><code>ipv6-cidr-block-association.state</code> - The state of an IPv6 CIDR block associated with the subnet.</p></li>
    /// <li>
    /// <p><code>ipv6-native</code> - Indicates whether this is an IPv6 only subnet (<code>true</code> | <code>false</code>).</p></li>
    /// <li>
    /// <p><code>map-customer-owned-ip-on-launch</code> - Indicates whether a network interface created in this subnet (including a network interface created by <code>RunInstances</code>) receives a customer-owned IPv4 address.</p></li>
    /// <li>
    /// <p><code>map-public-ip-on-launch</code> - Indicates whether instances launched in this subnet receive a public IPv4 address.</p></li>
    /// <li>
    /// <p><code>outpost-arn</code> - The Amazon Resource Name (ARN) of the Outpost.</p></li>
    /// <li>
    /// <p><code>owner-id</code> - The ID of the Amazon Web Services account that owns the subnet.</p></li>
    /// <li>
    /// <p><code>private-dns-name-options-on-launch.hostname-type</code> - The type of hostname to assign to instances in the subnet at launch. For IPv4-only and dual-stack (IPv4 and IPv6) subnets, an instance DNS name can be based on the instance IPv4 address (ip-name) or the instance ID (resource-name). For IPv6 only subnets, an instance DNS name must be based on the instance ID (resource-name).</p></li>
    /// <li>
    /// <p><code>private-dns-name-options-on-launch.enable-resource-name-dns-a-record</code> - Indicates whether to respond to DNS queries for instance hostnames with DNS A records.</p></li>
    /// <li>
    /// <p><code>private-dns-name-options-on-launch.enable-resource-name-dns-aaaa-record</code> - Indicates whether to respond to DNS queries for instance hostnames with DNS AAAA records.</p></li>
    /// <li>
    /// <p><code>state</code> - The state of the subnet (<code>pending</code> | <code>available</code>).</p></li>
    /// <li>
    /// <p><code>subnet-arn</code> - The Amazon Resource Name (ARN) of the subnet.</p></li>
    /// <li>
    /// <p><code>subnet-id</code> - The ID of the subnet.</p></li>
    /// <li>
    /// <p><code>tag</code> - The key/value combination of a tag assigned to the resource. Use the tag key in the filter name and the tag value as the filter value. For example, to find all resources that have a tag with the key <code>Owner</code> and the value <code>TeamA</code>, specify <code>tag:Owner</code> for the filter name and <code>TeamA</code> for the filter value.</p></li>
    /// <li>
    /// <p><code>tag-key</code> - The key of a tag assigned to the resource. Use this filter to find all resources assigned a tag with a specific key, regardless of the tag value.</p></li>
    /// <li>
    /// <p><code>vpc-id</code> - The ID of the VPC for the subnet.</p></li>
    /// </ul>
    pub fn set_filters(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Filter>>) -> Self {
        self.inner = self.inner.set_filters(input);
        self
    }
    /// <p>The filters.</p>
    /// <ul>
    /// <li>
    /// <p><code>availability-zone</code> - The Availability Zone for the subnet. You can also use <code>availabilityZone</code> as the filter name.</p></li>
    /// <li>
    /// <p><code>availability-zone-id</code> - The ID of the Availability Zone for the subnet. You can also use <code>availabilityZoneId</code> as the filter name.</p></li>
    /// <li>
    /// <p><code>available-ip-address-count</code> - The number of IPv4 addresses in the subnet that are available.</p></li>
    /// <li>
    /// <p><code>cidr-block</code> - The IPv4 CIDR block of the subnet. The CIDR block you specify must exactly match the subnet's CIDR block for information to be returned for the subnet. You can also use <code>cidr</code> or <code>cidrBlock</code> as the filter names.</p></li>
    /// <li>
    /// <p><code>customer-owned-ipv4-pool</code> - The customer-owned IPv4 address pool associated with the subnet.</p></li>
    /// <li>
    /// <p><code>default-for-az</code> - Indicates whether this is the default subnet for the Availability Zone (<code>true</code> | <code>false</code>). You can also use <code>defaultForAz</code> as the filter name.</p></li>
    /// <li>
    /// <p><code>enable-dns64</code> - Indicates whether DNS queries made to the Amazon-provided DNS Resolver in this subnet should return synthetic IPv6 addresses for IPv4-only destinations.</p></li>
    /// <li>
    /// <p><code>enable-lni-at-device-index</code> - Indicates the device position for local network interfaces in this subnet. For example, <code>1</code> indicates local network interfaces in this subnet are the secondary network interface (eth1).</p></li>
    /// <li>
    /// <p><code>ipv6-cidr-block-association.ipv6-cidr-block</code> - An IPv6 CIDR block associated with the subnet.</p></li>
    /// <li>
    /// <p><code>ipv6-cidr-block-association.association-id</code> - An association ID for an IPv6 CIDR block associated with the subnet.</p></li>
    /// <li>
    /// <p><code>ipv6-cidr-block-association.state</code> - The state of an IPv6 CIDR block associated with the subnet.</p></li>
    /// <li>
    /// <p><code>ipv6-native</code> - Indicates whether this is an IPv6 only subnet (<code>true</code> | <code>false</code>).</p></li>
    /// <li>
    /// <p><code>map-customer-owned-ip-on-launch</code> - Indicates whether a network interface created in this subnet (including a network interface created by <code>RunInstances</code>) receives a customer-owned IPv4 address.</p></li>
    /// <li>
    /// <p><code>map-public-ip-on-launch</code> - Indicates whether instances launched in this subnet receive a public IPv4 address.</p></li>
    /// <li>
    /// <p><code>outpost-arn</code> - The Amazon Resource Name (ARN) of the Outpost.</p></li>
    /// <li>
    /// <p><code>owner-id</code> - The ID of the Amazon Web Services account that owns the subnet.</p></li>
    /// <li>
    /// <p><code>private-dns-name-options-on-launch.hostname-type</code> - The type of hostname to assign to instances in the subnet at launch. For IPv4-only and dual-stack (IPv4 and IPv6) subnets, an instance DNS name can be based on the instance IPv4 address (ip-name) or the instance ID (resource-name). For IPv6 only subnets, an instance DNS name must be based on the instance ID (resource-name).</p></li>
    /// <li>
    /// <p><code>private-dns-name-options-on-launch.enable-resource-name-dns-a-record</code> - Indicates whether to respond to DNS queries for instance hostnames with DNS A records.</p></li>
    /// <li>
    /// <p><code>private-dns-name-options-on-launch.enable-resource-name-dns-aaaa-record</code> - Indicates whether to respond to DNS queries for instance hostnames with DNS AAAA records.</p></li>
    /// <li>
    /// <p><code>state</code> - The state of the subnet (<code>pending</code> | <code>available</code>).</p></li>
    /// <li>
    /// <p><code>subnet-arn</code> - The Amazon Resource Name (ARN) of the subnet.</p></li>
    /// <li>
    /// <p><code>subnet-id</code> - The ID of the subnet.</p></li>
    /// <li>
    /// <p><code>tag</code> - The key/value combination of a tag assigned to the resource. Use the tag key in the filter name and the tag value as the filter value. For example, to find all resources that have a tag with the key <code>Owner</code> and the value <code>TeamA</code>, specify <code>tag:Owner</code> for the filter name and <code>TeamA</code> for the filter value.</p></li>
    /// <li>
    /// <p><code>tag-key</code> - The key of a tag assigned to the resource. Use this filter to find all resources assigned a tag with a specific key, regardless of the tag value.</p></li>
    /// <li>
    /// <p><code>vpc-id</code> - The ID of the VPC for the subnet.</p></li>
    /// </ul>
    pub fn get_filters(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Filter>> {
        self.inner.get_filters()
    }
    ///
    /// Appends an item to `SubnetIds`.
    ///
    /// To override the contents of this collection use [`set_subnet_ids`](Self::set_subnet_ids).
    ///
    /// <p>The IDs of the subnets.</p>
    /// <p>Default: Describes all your subnets.</p>
    pub fn subnet_ids(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.subnet_ids(input.into());
        self
    }
    /// <p>The IDs of the subnets.</p>
    /// <p>Default: Describes all your subnets.</p>
    pub fn set_subnet_ids(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_subnet_ids(input);
        self
    }
    /// <p>The IDs of the subnets.</p>
    /// <p>Default: Describes all your subnets.</p>
    pub fn get_subnet_ids(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_subnet_ids()
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
}
