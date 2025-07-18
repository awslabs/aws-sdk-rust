// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DescribeFirewallMetadata`](crate::operation::describe_firewall_metadata::builders::DescribeFirewallMetadataFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`firewall_arn(impl Into<String>)`](crate::operation::describe_firewall_metadata::builders::DescribeFirewallMetadataFluentBuilder::firewall_arn) / [`set_firewall_arn(Option<String>)`](crate::operation::describe_firewall_metadata::builders::DescribeFirewallMetadataFluentBuilder::set_firewall_arn):<br>required: **false**<br><p>The Amazon Resource Name (ARN) of the firewall.</p><br>
    /// - On success, responds with [`DescribeFirewallMetadataOutput`](crate::operation::describe_firewall_metadata::DescribeFirewallMetadataOutput) with field(s):
    ///   - [`firewall_arn(Option<String>)`](crate::operation::describe_firewall_metadata::DescribeFirewallMetadataOutput::firewall_arn): <p>The Amazon Resource Name (ARN) of the firewall.</p>
    ///   - [`firewall_policy_arn(Option<String>)`](crate::operation::describe_firewall_metadata::DescribeFirewallMetadataOutput::firewall_policy_arn): <p>The Amazon Resource Name (ARN) of the firewall policy.</p>
    ///   - [`description(Option<String>)`](crate::operation::describe_firewall_metadata::DescribeFirewallMetadataOutput::description): <p>A description of the firewall.</p>
    ///   - [`status(Option<FirewallStatusValue>)`](crate::operation::describe_firewall_metadata::DescribeFirewallMetadataOutput::status): <p>The readiness of the configured firewall to handle network traffic across all of the Availability Zones where you have it configured. This setting is <code>READY</code> only when the <code>ConfigurationSyncStateSummary</code> value is <code>IN_SYNC</code> and the <code>Attachment</code> <code>Status</code> values for all of the configured subnets are <code>READY</code>.</p>
    ///   - [`supported_availability_zones(Option<HashMap::<String, AvailabilityZoneMetadata>>)`](crate::operation::describe_firewall_metadata::DescribeFirewallMetadataOutput::supported_availability_zones): <p>The Availability Zones that the firewall currently supports. This includes all Availability Zones for which the firewall has a subnet defined.</p>
    ///   - [`transit_gateway_attachment_id(Option<String>)`](crate::operation::describe_firewall_metadata::DescribeFirewallMetadataOutput::transit_gateway_attachment_id): <p>The unique identifier of the transit gateway attachment associated with this firewall. This field is only present for transit gateway-attached firewalls.</p>
    /// - On failure, responds with [`SdkError<DescribeFirewallMetadataError>`](crate::operation::describe_firewall_metadata::DescribeFirewallMetadataError)
    pub fn describe_firewall_metadata(&self) -> crate::operation::describe_firewall_metadata::builders::DescribeFirewallMetadataFluentBuilder {
        crate::operation::describe_firewall_metadata::builders::DescribeFirewallMetadataFluentBuilder::new(self.handle.clone())
    }
}
