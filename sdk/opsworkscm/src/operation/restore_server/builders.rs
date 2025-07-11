// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::restore_server::_restore_server_output::RestoreServerOutputBuilder;

pub use crate::operation::restore_server::_restore_server_input::RestoreServerInputBuilder;

impl crate::operation::restore_server::builders::RestoreServerInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::restore_server::RestoreServerOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::restore_server::RestoreServerError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.restore_server();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `RestoreServer`.
///
/// <p>Restores a backup to a server that is in a <code>CONNECTION_LOST</code>, <code>HEALTHY</code>, <code>RUNNING</code>, <code>UNHEALTHY</code>, or <code>TERMINATED</code> state. When you run RestoreServer, the server's EC2 instance is deleted, and a new EC2 instance is configured. RestoreServer maintains the existing server endpoint, so configuration management of the server's client devices (nodes) should continue to work.</p>
/// <p>Restoring from a backup is performed by creating a new EC2 instance. If restoration is successful, and the server is in a <code>HEALTHY</code> state, OpsWorks CM switches traffic over to the new instance. After restoration is finished, the old EC2 instance is maintained in a <code>Running</code> or <code>Stopped</code> state, but is eventually terminated.</p>
/// <p>This operation is asynchronous.</p>
/// <p>An <code>InvalidStateException</code> is thrown when the server is not in a valid state. A <code>ResourceNotFoundException</code> is thrown when the server does not exist. A <code>ValidationException</code> is raised when parameters of the request are not valid.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct RestoreServerFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::restore_server::builders::RestoreServerInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::restore_server::RestoreServerOutput,
        crate::operation::restore_server::RestoreServerError,
    > for RestoreServerFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::restore_server::RestoreServerOutput,
            crate::operation::restore_server::RestoreServerError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl RestoreServerFluentBuilder {
    /// Creates a new `RestoreServerFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the RestoreServer as a reference.
    pub fn as_input(&self) -> &crate::operation::restore_server::builders::RestoreServerInputBuilder {
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
        crate::operation::restore_server::RestoreServerOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::restore_server::RestoreServerError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::restore_server::RestoreServer::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::restore_server::RestoreServer::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::restore_server::RestoreServerOutput,
        crate::operation::restore_server::RestoreServerError,
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
    /// <p>The ID of the backup that you want to use to restore a server.</p>
    pub fn backup_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.backup_id(input.into());
        self
    }
    /// <p>The ID of the backup that you want to use to restore a server.</p>
    pub fn set_backup_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_backup_id(input);
        self
    }
    /// <p>The ID of the backup that you want to use to restore a server.</p>
    pub fn get_backup_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_backup_id()
    }
    /// <p>The name of the server that you want to restore.</p>
    pub fn server_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.server_name(input.into());
        self
    }
    /// <p>The name of the server that you want to restore.</p>
    pub fn set_server_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_server_name(input);
        self
    }
    /// <p>The name of the server that you want to restore.</p>
    pub fn get_server_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_server_name()
    }
    /// <p>The type of instance to restore. Valid values must be specified in the following format: <code>^(\[cm\]\[34\]|t2).*</code> For example, <code>m5.large</code>. Valid values are <code>m5.large</code>, <code>r5.xlarge</code>, and <code>r5.2xlarge</code>. If you do not specify this parameter, RestoreServer uses the instance type from the specified backup.</p>
    pub fn instance_type(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.instance_type(input.into());
        self
    }
    /// <p>The type of instance to restore. Valid values must be specified in the following format: <code>^(\[cm\]\[34\]|t2).*</code> For example, <code>m5.large</code>. Valid values are <code>m5.large</code>, <code>r5.xlarge</code>, and <code>r5.2xlarge</code>. If you do not specify this parameter, RestoreServer uses the instance type from the specified backup.</p>
    pub fn set_instance_type(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_instance_type(input);
        self
    }
    /// <p>The type of instance to restore. Valid values must be specified in the following format: <code>^(\[cm\]\[34\]|t2).*</code> For example, <code>m5.large</code>. Valid values are <code>m5.large</code>, <code>r5.xlarge</code>, and <code>r5.2xlarge</code>. If you do not specify this parameter, RestoreServer uses the instance type from the specified backup.</p>
    pub fn get_instance_type(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_instance_type()
    }
    /// <p>The name of the key pair to set on the new EC2 instance. This can be helpful if the administrator no longer has the SSH key.</p>
    pub fn key_pair(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.key_pair(input.into());
        self
    }
    /// <p>The name of the key pair to set on the new EC2 instance. This can be helpful if the administrator no longer has the SSH key.</p>
    pub fn set_key_pair(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_key_pair(input);
        self
    }
    /// <p>The name of the key pair to set on the new EC2 instance. This can be helpful if the administrator no longer has the SSH key.</p>
    pub fn get_key_pair(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_key_pair()
    }
}
