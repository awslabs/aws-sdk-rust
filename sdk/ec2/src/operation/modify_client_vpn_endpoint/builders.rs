// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::modify_client_vpn_endpoint::_modify_client_vpn_endpoint_output::ModifyClientVpnEndpointOutputBuilder;

pub use crate::operation::modify_client_vpn_endpoint::_modify_client_vpn_endpoint_input::ModifyClientVpnEndpointInputBuilder;

impl crate::operation::modify_client_vpn_endpoint::builders::ModifyClientVpnEndpointInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpointOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpointError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.modify_client_vpn_endpoint();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `ModifyClientVpnEndpoint`.
///
/// <p>Modifies the specified Client VPN endpoint. Modifying the DNS server resets existing client connections.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct ModifyClientVpnEndpointFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::modify_client_vpn_endpoint::builders::ModifyClientVpnEndpointInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpointOutput,
        crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpointError,
    > for ModifyClientVpnEndpointFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpointOutput,
            crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpointError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl ModifyClientVpnEndpointFluentBuilder {
    /// Creates a new `ModifyClientVpnEndpointFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the ModifyClientVpnEndpoint as a reference.
    pub fn as_input(&self) -> &crate::operation::modify_client_vpn_endpoint::builders::ModifyClientVpnEndpointInputBuilder {
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
        crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpointOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpointError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpoint::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpoint::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpointOutput,
        crate::operation::modify_client_vpn_endpoint::ModifyClientVpnEndpointError,
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
    /// <p>The ID of the Client VPN endpoint to modify.</p>
    pub fn client_vpn_endpoint_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_vpn_endpoint_id(input.into());
        self
    }
    /// <p>The ID of the Client VPN endpoint to modify.</p>
    pub fn set_client_vpn_endpoint_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_vpn_endpoint_id(input);
        self
    }
    /// <p>The ID of the Client VPN endpoint to modify.</p>
    pub fn get_client_vpn_endpoint_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_vpn_endpoint_id()
    }
    /// <p>The ARN of the server certificate to be used. The server certificate must be provisioned in Certificate Manager (ACM).</p>
    pub fn server_certificate_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.server_certificate_arn(input.into());
        self
    }
    /// <p>The ARN of the server certificate to be used. The server certificate must be provisioned in Certificate Manager (ACM).</p>
    pub fn set_server_certificate_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_server_certificate_arn(input);
        self
    }
    /// <p>The ARN of the server certificate to be used. The server certificate must be provisioned in Certificate Manager (ACM).</p>
    pub fn get_server_certificate_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_server_certificate_arn()
    }
    /// <p>Information about the client connection logging options.</p>
    /// <p>If you enable client connection logging, data about client connections is sent to a Cloudwatch Logs log stream. The following information is logged:</p>
    /// <ul>
    /// <li>
    /// <p>Client connection requests</p></li>
    /// <li>
    /// <p>Client connection results (successful and unsuccessful)</p></li>
    /// <li>
    /// <p>Reasons for unsuccessful client connection requests</p></li>
    /// <li>
    /// <p>Client connection termination time</p></li>
    /// </ul>
    pub fn connection_log_options(mut self, input: crate::types::ConnectionLogOptions) -> Self {
        self.inner = self.inner.connection_log_options(input);
        self
    }
    /// <p>Information about the client connection logging options.</p>
    /// <p>If you enable client connection logging, data about client connections is sent to a Cloudwatch Logs log stream. The following information is logged:</p>
    /// <ul>
    /// <li>
    /// <p>Client connection requests</p></li>
    /// <li>
    /// <p>Client connection results (successful and unsuccessful)</p></li>
    /// <li>
    /// <p>Reasons for unsuccessful client connection requests</p></li>
    /// <li>
    /// <p>Client connection termination time</p></li>
    /// </ul>
    pub fn set_connection_log_options(mut self, input: ::std::option::Option<crate::types::ConnectionLogOptions>) -> Self {
        self.inner = self.inner.set_connection_log_options(input);
        self
    }
    /// <p>Information about the client connection logging options.</p>
    /// <p>If you enable client connection logging, data about client connections is sent to a Cloudwatch Logs log stream. The following information is logged:</p>
    /// <ul>
    /// <li>
    /// <p>Client connection requests</p></li>
    /// <li>
    /// <p>Client connection results (successful and unsuccessful)</p></li>
    /// <li>
    /// <p>Reasons for unsuccessful client connection requests</p></li>
    /// <li>
    /// <p>Client connection termination time</p></li>
    /// </ul>
    pub fn get_connection_log_options(&self) -> &::std::option::Option<crate::types::ConnectionLogOptions> {
        self.inner.get_connection_log_options()
    }
    /// <p>Information about the DNS servers to be used by Client VPN connections. A Client VPN endpoint can have up to two DNS servers.</p>
    pub fn dns_servers(mut self, input: crate::types::DnsServersOptionsModifyStructure) -> Self {
        self.inner = self.inner.dns_servers(input);
        self
    }
    /// <p>Information about the DNS servers to be used by Client VPN connections. A Client VPN endpoint can have up to two DNS servers.</p>
    pub fn set_dns_servers(mut self, input: ::std::option::Option<crate::types::DnsServersOptionsModifyStructure>) -> Self {
        self.inner = self.inner.set_dns_servers(input);
        self
    }
    /// <p>Information about the DNS servers to be used by Client VPN connections. A Client VPN endpoint can have up to two DNS servers.</p>
    pub fn get_dns_servers(&self) -> &::std::option::Option<crate::types::DnsServersOptionsModifyStructure> {
        self.inner.get_dns_servers()
    }
    /// <p>The port number to assign to the Client VPN endpoint for TCP and UDP traffic.</p>
    /// <p>Valid Values: <code>443</code> | <code>1194</code></p>
    /// <p>Default Value: <code>443</code></p>
    pub fn vpn_port(mut self, input: i32) -> Self {
        self.inner = self.inner.vpn_port(input);
        self
    }
    /// <p>The port number to assign to the Client VPN endpoint for TCP and UDP traffic.</p>
    /// <p>Valid Values: <code>443</code> | <code>1194</code></p>
    /// <p>Default Value: <code>443</code></p>
    pub fn set_vpn_port(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_vpn_port(input);
        self
    }
    /// <p>The port number to assign to the Client VPN endpoint for TCP and UDP traffic.</p>
    /// <p>Valid Values: <code>443</code> | <code>1194</code></p>
    /// <p>Default Value: <code>443</code></p>
    pub fn get_vpn_port(&self) -> &::std::option::Option<i32> {
        self.inner.get_vpn_port()
    }
    /// <p>A brief description of the Client VPN endpoint.</p>
    pub fn description(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.description(input.into());
        self
    }
    /// <p>A brief description of the Client VPN endpoint.</p>
    pub fn set_description(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_description(input);
        self
    }
    /// <p>A brief description of the Client VPN endpoint.</p>
    pub fn get_description(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_description()
    }
    /// <p>Indicates whether the VPN is split-tunnel.</p>
    /// <p>For information about split-tunnel VPN endpoints, see <a href="https://docs.aws.amazon.com/vpn/latest/clientvpn-admin/split-tunnel-vpn.html">Split-tunnel Client VPN endpoint</a> in the <i>Client VPN Administrator Guide</i>.</p>
    pub fn split_tunnel(mut self, input: bool) -> Self {
        self.inner = self.inner.split_tunnel(input);
        self
    }
    /// <p>Indicates whether the VPN is split-tunnel.</p>
    /// <p>For information about split-tunnel VPN endpoints, see <a href="https://docs.aws.amazon.com/vpn/latest/clientvpn-admin/split-tunnel-vpn.html">Split-tunnel Client VPN endpoint</a> in the <i>Client VPN Administrator Guide</i>.</p>
    pub fn set_split_tunnel(mut self, input: ::std::option::Option<bool>) -> Self {
        self.inner = self.inner.set_split_tunnel(input);
        self
    }
    /// <p>Indicates whether the VPN is split-tunnel.</p>
    /// <p>For information about split-tunnel VPN endpoints, see <a href="https://docs.aws.amazon.com/vpn/latest/clientvpn-admin/split-tunnel-vpn.html">Split-tunnel Client VPN endpoint</a> in the <i>Client VPN Administrator Guide</i>.</p>
    pub fn get_split_tunnel(&self) -> &::std::option::Option<bool> {
        self.inner.get_split_tunnel()
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
    /// Appends an item to `SecurityGroupIds`.
    ///
    /// To override the contents of this collection use [`set_security_group_ids`](Self::set_security_group_ids).
    ///
    /// <p>The IDs of one or more security groups to apply to the target network.</p>
    pub fn security_group_ids(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.security_group_ids(input.into());
        self
    }
    /// <p>The IDs of one or more security groups to apply to the target network.</p>
    pub fn set_security_group_ids(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_security_group_ids(input);
        self
    }
    /// <p>The IDs of one or more security groups to apply to the target network.</p>
    pub fn get_security_group_ids(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_security_group_ids()
    }
    /// <p>The ID of the VPC to associate with the Client VPN endpoint.</p>
    pub fn vpc_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.vpc_id(input.into());
        self
    }
    /// <p>The ID of the VPC to associate with the Client VPN endpoint.</p>
    pub fn set_vpc_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_vpc_id(input);
        self
    }
    /// <p>The ID of the VPC to associate with the Client VPN endpoint.</p>
    pub fn get_vpc_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_vpc_id()
    }
    /// <p>Specify whether to enable the self-service portal for the Client VPN endpoint.</p>
    pub fn self_service_portal(mut self, input: crate::types::SelfServicePortal) -> Self {
        self.inner = self.inner.self_service_portal(input);
        self
    }
    /// <p>Specify whether to enable the self-service portal for the Client VPN endpoint.</p>
    pub fn set_self_service_portal(mut self, input: ::std::option::Option<crate::types::SelfServicePortal>) -> Self {
        self.inner = self.inner.set_self_service_portal(input);
        self
    }
    /// <p>Specify whether to enable the self-service portal for the Client VPN endpoint.</p>
    pub fn get_self_service_portal(&self) -> &::std::option::Option<crate::types::SelfServicePortal> {
        self.inner.get_self_service_portal()
    }
    /// <p>The options for managing connection authorization for new client connections.</p>
    pub fn client_connect_options(mut self, input: crate::types::ClientConnectOptions) -> Self {
        self.inner = self.inner.client_connect_options(input);
        self
    }
    /// <p>The options for managing connection authorization for new client connections.</p>
    pub fn set_client_connect_options(mut self, input: ::std::option::Option<crate::types::ClientConnectOptions>) -> Self {
        self.inner = self.inner.set_client_connect_options(input);
        self
    }
    /// <p>The options for managing connection authorization for new client connections.</p>
    pub fn get_client_connect_options(&self) -> &::std::option::Option<crate::types::ClientConnectOptions> {
        self.inner.get_client_connect_options()
    }
    /// <p>The maximum VPN session duration time in hours.</p>
    /// <p>Valid values: <code>8 | 10 | 12 | 24</code></p>
    /// <p>Default value: <code>24</code></p>
    pub fn session_timeout_hours(mut self, input: i32) -> Self {
        self.inner = self.inner.session_timeout_hours(input);
        self
    }
    /// <p>The maximum VPN session duration time in hours.</p>
    /// <p>Valid values: <code>8 | 10 | 12 | 24</code></p>
    /// <p>Default value: <code>24</code></p>
    pub fn set_session_timeout_hours(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_session_timeout_hours(input);
        self
    }
    /// <p>The maximum VPN session duration time in hours.</p>
    /// <p>Valid values: <code>8 | 10 | 12 | 24</code></p>
    /// <p>Default value: <code>24</code></p>
    pub fn get_session_timeout_hours(&self) -> &::std::option::Option<i32> {
        self.inner.get_session_timeout_hours()
    }
    /// <p>Options for enabling a customizable text banner that will be displayed on Amazon Web Services provided clients when a VPN session is established.</p>
    pub fn client_login_banner_options(mut self, input: crate::types::ClientLoginBannerOptions) -> Self {
        self.inner = self.inner.client_login_banner_options(input);
        self
    }
    /// <p>Options for enabling a customizable text banner that will be displayed on Amazon Web Services provided clients when a VPN session is established.</p>
    pub fn set_client_login_banner_options(mut self, input: ::std::option::Option<crate::types::ClientLoginBannerOptions>) -> Self {
        self.inner = self.inner.set_client_login_banner_options(input);
        self
    }
    /// <p>Options for enabling a customizable text banner that will be displayed on Amazon Web Services provided clients when a VPN session is established.</p>
    pub fn get_client_login_banner_options(&self) -> &::std::option::Option<crate::types::ClientLoginBannerOptions> {
        self.inner.get_client_login_banner_options()
    }
    /// <p>Client route enforcement is a feature of the Client VPN service that helps enforce administrator defined routes on devices connected through the VPN. T his feature helps improve your security posture by ensuring that network traffic originating from a connected client is not inadvertently sent outside the VPN tunnel.</p>
    /// <p>Client route enforcement works by monitoring the route table of a connected device for routing policy changes to the VPN connection. If the feature detects any VPN routing policy modifications, it will automatically force an update to the route table, reverting it back to the expected route configurations.</p>
    pub fn client_route_enforcement_options(mut self, input: crate::types::ClientRouteEnforcementOptions) -> Self {
        self.inner = self.inner.client_route_enforcement_options(input);
        self
    }
    /// <p>Client route enforcement is a feature of the Client VPN service that helps enforce administrator defined routes on devices connected through the VPN. T his feature helps improve your security posture by ensuring that network traffic originating from a connected client is not inadvertently sent outside the VPN tunnel.</p>
    /// <p>Client route enforcement works by monitoring the route table of a connected device for routing policy changes to the VPN connection. If the feature detects any VPN routing policy modifications, it will automatically force an update to the route table, reverting it back to the expected route configurations.</p>
    pub fn set_client_route_enforcement_options(mut self, input: ::std::option::Option<crate::types::ClientRouteEnforcementOptions>) -> Self {
        self.inner = self.inner.set_client_route_enforcement_options(input);
        self
    }
    /// <p>Client route enforcement is a feature of the Client VPN service that helps enforce administrator defined routes on devices connected through the VPN. T his feature helps improve your security posture by ensuring that network traffic originating from a connected client is not inadvertently sent outside the VPN tunnel.</p>
    /// <p>Client route enforcement works by monitoring the route table of a connected device for routing policy changes to the VPN connection. If the feature detects any VPN routing policy modifications, it will automatically force an update to the route table, reverting it back to the expected route configurations.</p>
    pub fn get_client_route_enforcement_options(&self) -> &::std::option::Option<crate::types::ClientRouteEnforcementOptions> {
        self.inner.get_client_route_enforcement_options()
    }
    /// <p>Indicates whether the client VPN session is disconnected after the maximum timeout specified in <code>sessionTimeoutHours</code> is reached. If <code>true</code>, users are prompted to reconnect client VPN. If <code>false</code>, client VPN attempts to reconnect automatically. The default value is <code>true</code>.</p>
    pub fn disconnect_on_session_timeout(mut self, input: bool) -> Self {
        self.inner = self.inner.disconnect_on_session_timeout(input);
        self
    }
    /// <p>Indicates whether the client VPN session is disconnected after the maximum timeout specified in <code>sessionTimeoutHours</code> is reached. If <code>true</code>, users are prompted to reconnect client VPN. If <code>false</code>, client VPN attempts to reconnect automatically. The default value is <code>true</code>.</p>
    pub fn set_disconnect_on_session_timeout(mut self, input: ::std::option::Option<bool>) -> Self {
        self.inner = self.inner.set_disconnect_on_session_timeout(input);
        self
    }
    /// <p>Indicates whether the client VPN session is disconnected after the maximum timeout specified in <code>sessionTimeoutHours</code> is reached. If <code>true</code>, users are prompted to reconnect client VPN. If <code>false</code>, client VPN attempts to reconnect automatically. The default value is <code>true</code>.</p>
    pub fn get_disconnect_on_session_timeout(&self) -> &::std::option::Option<bool> {
        self.inner.get_disconnect_on_session_timeout()
    }
}
