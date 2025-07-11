// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::get_code_security_scan::_get_code_security_scan_output::GetCodeSecurityScanOutputBuilder;

pub use crate::operation::get_code_security_scan::_get_code_security_scan_input::GetCodeSecurityScanInputBuilder;

impl crate::operation::get_code_security_scan::builders::GetCodeSecurityScanInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::get_code_security_scan::GetCodeSecurityScanOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::get_code_security_scan::GetCodeSecurityScanError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.get_code_security_scan();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `GetCodeSecurityScan`.
///
/// <p>Retrieves information about a specific code security scan.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct GetCodeSecurityScanFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::get_code_security_scan::builders::GetCodeSecurityScanInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::get_code_security_scan::GetCodeSecurityScanOutput,
        crate::operation::get_code_security_scan::GetCodeSecurityScanError,
    > for GetCodeSecurityScanFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::get_code_security_scan::GetCodeSecurityScanOutput,
            crate::operation::get_code_security_scan::GetCodeSecurityScanError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl GetCodeSecurityScanFluentBuilder {
    /// Creates a new `GetCodeSecurityScanFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the GetCodeSecurityScan as a reference.
    pub fn as_input(&self) -> &crate::operation::get_code_security_scan::builders::GetCodeSecurityScanInputBuilder {
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
        crate::operation::get_code_security_scan::GetCodeSecurityScanOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::get_code_security_scan::GetCodeSecurityScanError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::get_code_security_scan::GetCodeSecurityScan::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::get_code_security_scan::GetCodeSecurityScan::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::get_code_security_scan::GetCodeSecurityScanOutput,
        crate::operation::get_code_security_scan::GetCodeSecurityScanError,
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
    /// <p>The resource identifier for the code repository that was scanned.</p>
    pub fn resource(mut self, input: crate::types::CodeSecurityResource) -> Self {
        self.inner = self.inner.resource(input);
        self
    }
    /// <p>The resource identifier for the code repository that was scanned.</p>
    pub fn set_resource(mut self, input: ::std::option::Option<crate::types::CodeSecurityResource>) -> Self {
        self.inner = self.inner.set_resource(input);
        self
    }
    /// <p>The resource identifier for the code repository that was scanned.</p>
    pub fn get_resource(&self) -> &::std::option::Option<crate::types::CodeSecurityResource> {
        self.inner.get_resource()
    }
    /// <p>The unique identifier of the scan to retrieve.</p>
    pub fn scan_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.scan_id(input.into());
        self
    }
    /// <p>The unique identifier of the scan to retrieve.</p>
    pub fn set_scan_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_scan_id(input);
        self
    }
    /// <p>The unique identifier of the scan to retrieve.</p>
    pub fn get_scan_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_scan_id()
    }
}
