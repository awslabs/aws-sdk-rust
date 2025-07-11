// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_customer_gateway::_create_customer_gateway_output::CreateCustomerGatewayOutputBuilder;

pub use crate::operation::create_customer_gateway::_create_customer_gateway_input::CreateCustomerGatewayInputBuilder;

impl crate::operation::create_customer_gateway::builders::CreateCustomerGatewayInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_customer_gateway::CreateCustomerGatewayOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_customer_gateway::CreateCustomerGatewayError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_customer_gateway();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateCustomerGateway`.
///
/// <p>Provides information to Amazon Web Services about your customer gateway device. The customer gateway device is the appliance at your end of the VPN connection. You must provide the IP address of the customer gateway device’s external interface. The IP address must be static and can be behind a device performing network address translation (NAT).</p>
/// <p>For devices that use Border Gateway Protocol (BGP), you can also provide the device's BGP Autonomous System Number (ASN). You can use an existing ASN assigned to your network. If you don't have an ASN already, you can use a private ASN. For more information, see <a href="https://docs.aws.amazon.com/vpn/latest/s2svpn/cgw-options.html">Customer gateway options for your Site-to-Site VPN connection</a> in the <i>Amazon Web Services Site-to-Site VPN User Guide</i>.</p>
/// <p>To create more than one customer gateway with the same VPN type, IP address, and BGP ASN, specify a unique device name for each customer gateway. An identical request returns information about the existing customer gateway; it doesn't create a new customer gateway.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateCustomerGatewayFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_customer_gateway::builders::CreateCustomerGatewayInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_customer_gateway::CreateCustomerGatewayOutput,
        crate::operation::create_customer_gateway::CreateCustomerGatewayError,
    > for CreateCustomerGatewayFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_customer_gateway::CreateCustomerGatewayOutput,
            crate::operation::create_customer_gateway::CreateCustomerGatewayError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateCustomerGatewayFluentBuilder {
    /// Creates a new `CreateCustomerGatewayFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateCustomerGateway as a reference.
    pub fn as_input(&self) -> &crate::operation::create_customer_gateway::builders::CreateCustomerGatewayInputBuilder {
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
        crate::operation::create_customer_gateway::CreateCustomerGatewayOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_customer_gateway::CreateCustomerGatewayError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_customer_gateway::CreateCustomerGateway::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_customer_gateway::CreateCustomerGateway::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_customer_gateway::CreateCustomerGatewayOutput,
        crate::operation::create_customer_gateway::CreateCustomerGatewayError,
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
    /// <p>For customer gateway devices that support BGP, specify the device's ASN. You must specify either <code>BgpAsn</code> or <code>BgpAsnExtended</code> when creating the customer gateway. If the ASN is larger than <code>2,147,483,647</code>, you must use <code>BgpAsnExtended</code>.</p>
    /// <p>Default: 65000</p>
    /// <p>Valid values: <code>1</code> to <code>2,147,483,647</code></p>
    pub fn bgp_asn(mut self, input: i32) -> Self {
        self.inner = self.inner.bgp_asn(input);
        self
    }
    /// <p>For customer gateway devices that support BGP, specify the device's ASN. You must specify either <code>BgpAsn</code> or <code>BgpAsnExtended</code> when creating the customer gateway. If the ASN is larger than <code>2,147,483,647</code>, you must use <code>BgpAsnExtended</code>.</p>
    /// <p>Default: 65000</p>
    /// <p>Valid values: <code>1</code> to <code>2,147,483,647</code></p>
    pub fn set_bgp_asn(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_bgp_asn(input);
        self
    }
    /// <p>For customer gateway devices that support BGP, specify the device's ASN. You must specify either <code>BgpAsn</code> or <code>BgpAsnExtended</code> when creating the customer gateway. If the ASN is larger than <code>2,147,483,647</code>, you must use <code>BgpAsnExtended</code>.</p>
    /// <p>Default: 65000</p>
    /// <p>Valid values: <code>1</code> to <code>2,147,483,647</code></p>
    pub fn get_bgp_asn(&self) -> &::std::option::Option<i32> {
        self.inner.get_bgp_asn()
    }
    /// <p><i>This member has been deprecated.</i> The Internet-routable IP address for the customer gateway's outside interface. The address must be static.</p>
    pub fn public_ip(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.public_ip(input.into());
        self
    }
    /// <p><i>This member has been deprecated.</i> The Internet-routable IP address for the customer gateway's outside interface. The address must be static.</p>
    pub fn set_public_ip(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_public_ip(input);
        self
    }
    /// <p><i>This member has been deprecated.</i> The Internet-routable IP address for the customer gateway's outside interface. The address must be static.</p>
    pub fn get_public_ip(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_public_ip()
    }
    /// <p>The Amazon Resource Name (ARN) for the customer gateway certificate.</p>
    pub fn certificate_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.certificate_arn(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) for the customer gateway certificate.</p>
    pub fn set_certificate_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_certificate_arn(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) for the customer gateway certificate.</p>
    pub fn get_certificate_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_certificate_arn()
    }
    /// <p>The type of VPN connection that this customer gateway supports (<code>ipsec.1</code>).</p>
    pub fn r#type(mut self, input: crate::types::GatewayType) -> Self {
        self.inner = self.inner.r#type(input);
        self
    }
    /// <p>The type of VPN connection that this customer gateway supports (<code>ipsec.1</code>).</p>
    pub fn set_type(mut self, input: ::std::option::Option<crate::types::GatewayType>) -> Self {
        self.inner = self.inner.set_type(input);
        self
    }
    /// <p>The type of VPN connection that this customer gateway supports (<code>ipsec.1</code>).</p>
    pub fn get_type(&self) -> &::std::option::Option<crate::types::GatewayType> {
        self.inner.get_type()
    }
    ///
    /// Appends an item to `TagSpecifications`.
    ///
    /// To override the contents of this collection use [`set_tag_specifications`](Self::set_tag_specifications).
    ///
    /// <p>The tags to apply to the customer gateway.</p>
    pub fn tag_specifications(mut self, input: crate::types::TagSpecification) -> Self {
        self.inner = self.inner.tag_specifications(input);
        self
    }
    /// <p>The tags to apply to the customer gateway.</p>
    pub fn set_tag_specifications(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::TagSpecification>>) -> Self {
        self.inner = self.inner.set_tag_specifications(input);
        self
    }
    /// <p>The tags to apply to the customer gateway.</p>
    pub fn get_tag_specifications(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::TagSpecification>> {
        self.inner.get_tag_specifications()
    }
    /// <p>A name for the customer gateway device.</p>
    /// <p>Length Constraints: Up to 255 characters.</p>
    pub fn device_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.device_name(input.into());
        self
    }
    /// <p>A name for the customer gateway device.</p>
    /// <p>Length Constraints: Up to 255 characters.</p>
    pub fn set_device_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_device_name(input);
        self
    }
    /// <p>A name for the customer gateway device.</p>
    /// <p>Length Constraints: Up to 255 characters.</p>
    pub fn get_device_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_device_name()
    }
    /// <p>The IP address for the customer gateway device's outside interface. The address must be static. If <code>OutsideIpAddressType</code> in your VPN connection options is set to <code>PrivateIpv4</code>, you can use an RFC6598 or RFC1918 private IPv4 address. If <code>OutsideIpAddressType</code> is set to <code>Ipv6</code>, you can use an IPv6 address.</p>
    pub fn ip_address(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.ip_address(input.into());
        self
    }
    /// <p>The IP address for the customer gateway device's outside interface. The address must be static. If <code>OutsideIpAddressType</code> in your VPN connection options is set to <code>PrivateIpv4</code>, you can use an RFC6598 or RFC1918 private IPv4 address. If <code>OutsideIpAddressType</code> is set to <code>Ipv6</code>, you can use an IPv6 address.</p>
    pub fn set_ip_address(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_ip_address(input);
        self
    }
    /// <p>The IP address for the customer gateway device's outside interface. The address must be static. If <code>OutsideIpAddressType</code> in your VPN connection options is set to <code>PrivateIpv4</code>, you can use an RFC6598 or RFC1918 private IPv4 address. If <code>OutsideIpAddressType</code> is set to <code>Ipv6</code>, you can use an IPv6 address.</p>
    pub fn get_ip_address(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_ip_address()
    }
    /// <p>For customer gateway devices that support BGP, specify the device's ASN. You must specify either <code>BgpAsn</code> or <code>BgpAsnExtended</code> when creating the customer gateway. If the ASN is larger than <code>2,147,483,647</code>, you must use <code>BgpAsnExtended</code>.</p>
    /// <p>Valid values: <code>2,147,483,648</code> to <code>4,294,967,295</code></p>
    pub fn bgp_asn_extended(mut self, input: i64) -> Self {
        self.inner = self.inner.bgp_asn_extended(input);
        self
    }
    /// <p>For customer gateway devices that support BGP, specify the device's ASN. You must specify either <code>BgpAsn</code> or <code>BgpAsnExtended</code> when creating the customer gateway. If the ASN is larger than <code>2,147,483,647</code>, you must use <code>BgpAsnExtended</code>.</p>
    /// <p>Valid values: <code>2,147,483,648</code> to <code>4,294,967,295</code></p>
    pub fn set_bgp_asn_extended(mut self, input: ::std::option::Option<i64>) -> Self {
        self.inner = self.inner.set_bgp_asn_extended(input);
        self
    }
    /// <p>For customer gateway devices that support BGP, specify the device's ASN. You must specify either <code>BgpAsn</code> or <code>BgpAsnExtended</code> when creating the customer gateway. If the ASN is larger than <code>2,147,483,647</code>, you must use <code>BgpAsnExtended</code>.</p>
    /// <p>Valid values: <code>2,147,483,648</code> to <code>4,294,967,295</code></p>
    pub fn get_bgp_asn_extended(&self) -> &::std::option::Option<i64> {
        self.inner.get_bgp_asn_extended()
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
