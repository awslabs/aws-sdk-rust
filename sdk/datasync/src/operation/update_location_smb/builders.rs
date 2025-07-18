// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::update_location_smb::_update_location_smb_output::UpdateLocationSmbOutputBuilder;

pub use crate::operation::update_location_smb::_update_location_smb_input::UpdateLocationSmbInputBuilder;

impl crate::operation::update_location_smb::builders::UpdateLocationSmbInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::update_location_smb::UpdateLocationSmbOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_location_smb::UpdateLocationSmbError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.update_location_smb();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UpdateLocationSmb`.
///
/// <p>Modifies the following configuration parameters of the Server Message Block (SMB) transfer location that you're using with DataSync.</p>
/// <p>For more information, see <a href="https://docs.aws.amazon.com/datasync/latest/userguide/create-smb-location.html">Configuring DataSync transfers with an SMB file server</a>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UpdateLocationSmbFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::update_location_smb::builders::UpdateLocationSmbInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::update_location_smb::UpdateLocationSmbOutput,
        crate::operation::update_location_smb::UpdateLocationSmbError,
    > for UpdateLocationSmbFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::update_location_smb::UpdateLocationSmbOutput,
            crate::operation::update_location_smb::UpdateLocationSmbError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UpdateLocationSmbFluentBuilder {
    /// Creates a new `UpdateLocationSmbFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UpdateLocationSmb as a reference.
    pub fn as_input(&self) -> &crate::operation::update_location_smb::builders::UpdateLocationSmbInputBuilder {
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
        crate::operation::update_location_smb::UpdateLocationSmbOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_location_smb::UpdateLocationSmbError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::update_location_smb::UpdateLocationSmb::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::update_location_smb::UpdateLocationSmb::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::update_location_smb::UpdateLocationSmbOutput,
        crate::operation::update_location_smb::UpdateLocationSmbError,
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
    /// <p>Specifies the ARN of the SMB location that you want to update.</p>
    pub fn location_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.location_arn(input.into());
        self
    }
    /// <p>Specifies the ARN of the SMB location that you want to update.</p>
    pub fn set_location_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_location_arn(input);
        self
    }
    /// <p>Specifies the ARN of the SMB location that you want to update.</p>
    pub fn get_location_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_location_arn()
    }
    /// <p>Specifies the name of the share exported by your SMB file server where DataSync will read or write data. You can include a subdirectory in the share path (for example, <code>/path/to/subdirectory</code>). Make sure that other SMB clients in your network can also mount this path.</p>
    /// <p>To copy all data in the specified subdirectory, DataSync must be able to mount the SMB share and access all of its data. For more information, see <a href="https://docs.aws.amazon.com/datasync/latest/userguide/create-smb-location.html#configuring-smb-permissions">Providing DataSync access to SMB file servers</a>.</p>
    pub fn subdirectory(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.subdirectory(input.into());
        self
    }
    /// <p>Specifies the name of the share exported by your SMB file server where DataSync will read or write data. You can include a subdirectory in the share path (for example, <code>/path/to/subdirectory</code>). Make sure that other SMB clients in your network can also mount this path.</p>
    /// <p>To copy all data in the specified subdirectory, DataSync must be able to mount the SMB share and access all of its data. For more information, see <a href="https://docs.aws.amazon.com/datasync/latest/userguide/create-smb-location.html#configuring-smb-permissions">Providing DataSync access to SMB file servers</a>.</p>
    pub fn set_subdirectory(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_subdirectory(input);
        self
    }
    /// <p>Specifies the name of the share exported by your SMB file server where DataSync will read or write data. You can include a subdirectory in the share path (for example, <code>/path/to/subdirectory</code>). Make sure that other SMB clients in your network can also mount this path.</p>
    /// <p>To copy all data in the specified subdirectory, DataSync must be able to mount the SMB share and access all of its data. For more information, see <a href="https://docs.aws.amazon.com/datasync/latest/userguide/create-smb-location.html#configuring-smb-permissions">Providing DataSync access to SMB file servers</a>.</p>
    pub fn get_subdirectory(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_subdirectory()
    }
    /// <p>Specifies the domain name or IP address (IPv4 or IPv6) of the SMB file server that your DataSync agent connects to.</p><note>
    /// <p>If you're using Kerberos authentication, you must specify a domain name.</p>
    /// </note>
    pub fn server_hostname(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.server_hostname(input.into());
        self
    }
    /// <p>Specifies the domain name or IP address (IPv4 or IPv6) of the SMB file server that your DataSync agent connects to.</p><note>
    /// <p>If you're using Kerberos authentication, you must specify a domain name.</p>
    /// </note>
    pub fn set_server_hostname(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_server_hostname(input);
        self
    }
    /// <p>Specifies the domain name or IP address (IPv4 or IPv6) of the SMB file server that your DataSync agent connects to.</p><note>
    /// <p>If you're using Kerberos authentication, you must specify a domain name.</p>
    /// </note>
    pub fn get_server_hostname(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_server_hostname()
    }
    /// <p>Specifies the user name that can mount your SMB file server and has permission to access the files and folders involved in your transfer. This parameter applies only if <code>AuthenticationType</code> is set to <code>NTLM</code>.</p>
    /// <p>For information about choosing a user with the right level of access for your transfer, see <a href="https://docs.aws.amazon.com/datasync/latest/userguide/create-smb-location.html#configuring-smb-permissions">Providing DataSync access to SMB file servers</a>.</p>
    pub fn user(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.user(input.into());
        self
    }
    /// <p>Specifies the user name that can mount your SMB file server and has permission to access the files and folders involved in your transfer. This parameter applies only if <code>AuthenticationType</code> is set to <code>NTLM</code>.</p>
    /// <p>For information about choosing a user with the right level of access for your transfer, see <a href="https://docs.aws.amazon.com/datasync/latest/userguide/create-smb-location.html#configuring-smb-permissions">Providing DataSync access to SMB file servers</a>.</p>
    pub fn set_user(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_user(input);
        self
    }
    /// <p>Specifies the user name that can mount your SMB file server and has permission to access the files and folders involved in your transfer. This parameter applies only if <code>AuthenticationType</code> is set to <code>NTLM</code>.</p>
    /// <p>For information about choosing a user with the right level of access for your transfer, see <a href="https://docs.aws.amazon.com/datasync/latest/userguide/create-smb-location.html#configuring-smb-permissions">Providing DataSync access to SMB file servers</a>.</p>
    pub fn get_user(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_user()
    }
    /// <p>Specifies the Windows domain name that your SMB file server belongs to. This parameter applies only if <code>AuthenticationType</code> is set to <code>NTLM</code>.</p>
    /// <p>If you have multiple domains in your environment, configuring this parameter makes sure that DataSync connects to the right file server.</p>
    pub fn domain(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.domain(input.into());
        self
    }
    /// <p>Specifies the Windows domain name that your SMB file server belongs to. This parameter applies only if <code>AuthenticationType</code> is set to <code>NTLM</code>.</p>
    /// <p>If you have multiple domains in your environment, configuring this parameter makes sure that DataSync connects to the right file server.</p>
    pub fn set_domain(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_domain(input);
        self
    }
    /// <p>Specifies the Windows domain name that your SMB file server belongs to. This parameter applies only if <code>AuthenticationType</code> is set to <code>NTLM</code>.</p>
    /// <p>If you have multiple domains in your environment, configuring this parameter makes sure that DataSync connects to the right file server.</p>
    pub fn get_domain(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_domain()
    }
    /// <p>Specifies the password of the user who can mount your SMB file server and has permission to access the files and folders involved in your transfer. This parameter applies only if <code>AuthenticationType</code> is set to <code>NTLM</code>.</p>
    pub fn password(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.password(input.into());
        self
    }
    /// <p>Specifies the password of the user who can mount your SMB file server and has permission to access the files and folders involved in your transfer. This parameter applies only if <code>AuthenticationType</code> is set to <code>NTLM</code>.</p>
    pub fn set_password(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_password(input);
        self
    }
    /// <p>Specifies the password of the user who can mount your SMB file server and has permission to access the files and folders involved in your transfer. This parameter applies only if <code>AuthenticationType</code> is set to <code>NTLM</code>.</p>
    pub fn get_password(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_password()
    }
    ///
    /// Appends an item to `AgentArns`.
    ///
    /// To override the contents of this collection use [`set_agent_arns`](Self::set_agent_arns).
    ///
    /// <p>Specifies the DataSync agent (or agents) that can connect to your SMB file server. You specify an agent by using its Amazon Resource Name (ARN).</p>
    pub fn agent_arns(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.agent_arns(input.into());
        self
    }
    /// <p>Specifies the DataSync agent (or agents) that can connect to your SMB file server. You specify an agent by using its Amazon Resource Name (ARN).</p>
    pub fn set_agent_arns(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_agent_arns(input);
        self
    }
    /// <p>Specifies the DataSync agent (or agents) that can connect to your SMB file server. You specify an agent by using its Amazon Resource Name (ARN).</p>
    pub fn get_agent_arns(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_agent_arns()
    }
    /// <p>Specifies the version of the Server Message Block (SMB) protocol that DataSync uses to access an SMB file server.</p>
    pub fn mount_options(mut self, input: crate::types::SmbMountOptions) -> Self {
        self.inner = self.inner.mount_options(input);
        self
    }
    /// <p>Specifies the version of the Server Message Block (SMB) protocol that DataSync uses to access an SMB file server.</p>
    pub fn set_mount_options(mut self, input: ::std::option::Option<crate::types::SmbMountOptions>) -> Self {
        self.inner = self.inner.set_mount_options(input);
        self
    }
    /// <p>Specifies the version of the Server Message Block (SMB) protocol that DataSync uses to access an SMB file server.</p>
    pub fn get_mount_options(&self) -> &::std::option::Option<crate::types::SmbMountOptions> {
        self.inner.get_mount_options()
    }
    /// <p>Specifies the authentication protocol that DataSync uses to connect to your SMB file server. DataSync supports <code>NTLM</code> (default) and <code>KERBEROS</code> authentication.</p>
    /// <p>For more information, see <a href="https://docs.aws.amazon.com/datasync/latest/userguide/create-smb-location.html#configuring-smb-permissions">Providing DataSync access to SMB file servers</a>.</p>
    pub fn authentication_type(mut self, input: crate::types::SmbAuthenticationType) -> Self {
        self.inner = self.inner.authentication_type(input);
        self
    }
    /// <p>Specifies the authentication protocol that DataSync uses to connect to your SMB file server. DataSync supports <code>NTLM</code> (default) and <code>KERBEROS</code> authentication.</p>
    /// <p>For more information, see <a href="https://docs.aws.amazon.com/datasync/latest/userguide/create-smb-location.html#configuring-smb-permissions">Providing DataSync access to SMB file servers</a>.</p>
    pub fn set_authentication_type(mut self, input: ::std::option::Option<crate::types::SmbAuthenticationType>) -> Self {
        self.inner = self.inner.set_authentication_type(input);
        self
    }
    /// <p>Specifies the authentication protocol that DataSync uses to connect to your SMB file server. DataSync supports <code>NTLM</code> (default) and <code>KERBEROS</code> authentication.</p>
    /// <p>For more information, see <a href="https://docs.aws.amazon.com/datasync/latest/userguide/create-smb-location.html#configuring-smb-permissions">Providing DataSync access to SMB file servers</a>.</p>
    pub fn get_authentication_type(&self) -> &::std::option::Option<crate::types::SmbAuthenticationType> {
        self.inner.get_authentication_type()
    }
    ///
    /// Appends an item to `DnsIpAddresses`.
    ///
    /// To override the contents of this collection use [`set_dns_ip_addresses`](Self::set_dns_ip_addresses).
    ///
    /// <p>Specifies the IP addresses (IPv4 or IPv6) for the DNS servers that your SMB file server belongs to. This parameter applies only if <code>AuthenticationType</code> is set to <code>KERBEROS</code>.</p>
    /// <p>If you have multiple domains in your environment, configuring this parameter makes sure that DataSync connects to the right SMB file server.</p>
    pub fn dns_ip_addresses(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.dns_ip_addresses(input.into());
        self
    }
    /// <p>Specifies the IP addresses (IPv4 or IPv6) for the DNS servers that your SMB file server belongs to. This parameter applies only if <code>AuthenticationType</code> is set to <code>KERBEROS</code>.</p>
    /// <p>If you have multiple domains in your environment, configuring this parameter makes sure that DataSync connects to the right SMB file server.</p>
    pub fn set_dns_ip_addresses(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_dns_ip_addresses(input);
        self
    }
    /// <p>Specifies the IP addresses (IPv4 or IPv6) for the DNS servers that your SMB file server belongs to. This parameter applies only if <code>AuthenticationType</code> is set to <code>KERBEROS</code>.</p>
    /// <p>If you have multiple domains in your environment, configuring this parameter makes sure that DataSync connects to the right SMB file server.</p>
    pub fn get_dns_ip_addresses(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_dns_ip_addresses()
    }
    /// <p>Specifies a Kerberos prinicpal, which is an identity in your Kerberos realm that has permission to access the files, folders, and file metadata in your SMB file server.</p>
    /// <p>A Kerberos principal might look like <code>HOST/kerberosuser@MYDOMAIN.ORG</code>.</p>
    /// <p>Principal names are case sensitive. Your DataSync task execution will fail if the principal that you specify for this parameter doesn’t exactly match the principal that you use to create the keytab file.</p>
    pub fn kerberos_principal(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.kerberos_principal(input.into());
        self
    }
    /// <p>Specifies a Kerberos prinicpal, which is an identity in your Kerberos realm that has permission to access the files, folders, and file metadata in your SMB file server.</p>
    /// <p>A Kerberos principal might look like <code>HOST/kerberosuser@MYDOMAIN.ORG</code>.</p>
    /// <p>Principal names are case sensitive. Your DataSync task execution will fail if the principal that you specify for this parameter doesn’t exactly match the principal that you use to create the keytab file.</p>
    pub fn set_kerberos_principal(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_kerberos_principal(input);
        self
    }
    /// <p>Specifies a Kerberos prinicpal, which is an identity in your Kerberos realm that has permission to access the files, folders, and file metadata in your SMB file server.</p>
    /// <p>A Kerberos principal might look like <code>HOST/kerberosuser@MYDOMAIN.ORG</code>.</p>
    /// <p>Principal names are case sensitive. Your DataSync task execution will fail if the principal that you specify for this parameter doesn’t exactly match the principal that you use to create the keytab file.</p>
    pub fn get_kerberos_principal(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_kerberos_principal()
    }
    /// <p>Specifies your Kerberos key table (keytab) file, which includes mappings between your Kerberos principal and encryption keys.</p>
    /// <p>To avoid task execution errors, make sure that the Kerberos principal that you use to create the keytab file matches exactly what you specify for <code>KerberosPrincipal</code>.</p>
    pub fn kerberos_keytab(mut self, input: ::aws_smithy_types::Blob) -> Self {
        self.inner = self.inner.kerberos_keytab(input);
        self
    }
    /// <p>Specifies your Kerberos key table (keytab) file, which includes mappings between your Kerberos principal and encryption keys.</p>
    /// <p>To avoid task execution errors, make sure that the Kerberos principal that you use to create the keytab file matches exactly what you specify for <code>KerberosPrincipal</code>.</p>
    pub fn set_kerberos_keytab(mut self, input: ::std::option::Option<::aws_smithy_types::Blob>) -> Self {
        self.inner = self.inner.set_kerberos_keytab(input);
        self
    }
    /// <p>Specifies your Kerberos key table (keytab) file, which includes mappings between your Kerberos principal and encryption keys.</p>
    /// <p>To avoid task execution errors, make sure that the Kerberos principal that you use to create the keytab file matches exactly what you specify for <code>KerberosPrincipal</code>.</p>
    pub fn get_kerberos_keytab(&self) -> &::std::option::Option<::aws_smithy_types::Blob> {
        self.inner.get_kerberos_keytab()
    }
    /// <p>Specifies a Kerberos configuration file (<code>krb5.conf</code>) that defines your Kerberos realm configuration.</p>
    /// <p>The file must be base64 encoded. If you're using the CLI, the encoding is done for you.</p>
    pub fn kerberos_krb5_conf(mut self, input: ::aws_smithy_types::Blob) -> Self {
        self.inner = self.inner.kerberos_krb5_conf(input);
        self
    }
    /// <p>Specifies a Kerberos configuration file (<code>krb5.conf</code>) that defines your Kerberos realm configuration.</p>
    /// <p>The file must be base64 encoded. If you're using the CLI, the encoding is done for you.</p>
    pub fn set_kerberos_krb5_conf(mut self, input: ::std::option::Option<::aws_smithy_types::Blob>) -> Self {
        self.inner = self.inner.set_kerberos_krb5_conf(input);
        self
    }
    /// <p>Specifies a Kerberos configuration file (<code>krb5.conf</code>) that defines your Kerberos realm configuration.</p>
    /// <p>The file must be base64 encoded. If you're using the CLI, the encoding is done for you.</p>
    pub fn get_kerberos_krb5_conf(&self) -> &::std::option::Option<::aws_smithy_types::Blob> {
        self.inner.get_kerberos_krb5_conf()
    }
}
