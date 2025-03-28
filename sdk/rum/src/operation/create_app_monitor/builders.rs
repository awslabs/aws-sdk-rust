// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_app_monitor::_create_app_monitor_output::CreateAppMonitorOutputBuilder;

pub use crate::operation::create_app_monitor::_create_app_monitor_input::CreateAppMonitorInputBuilder;

impl crate::operation::create_app_monitor::builders::CreateAppMonitorInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_app_monitor::CreateAppMonitorOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_app_monitor::CreateAppMonitorError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_app_monitor();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateAppMonitor`.
///
/// <p>Creates a Amazon CloudWatch RUM app monitor, which collects telemetry data from your application and sends that data to RUM. The data includes performance and reliability information such as page load time, client-side errors, and user behavior.</p>
/// <p>You use this operation only to create a new app monitor. To update an existing app monitor, use <a href="https://docs.aws.amazon.com/cloudwatchrum/latest/APIReference/API_UpdateAppMonitor.html">UpdateAppMonitor</a> instead.</p>
/// <p>After you create an app monitor, sign in to the CloudWatch RUM console to get the JavaScript code snippet to add to your web application. For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-RUM-find-code-snippet.html">How do I find a code snippet that I've already generated?</a></p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateAppMonitorFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_app_monitor::builders::CreateAppMonitorInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_app_monitor::CreateAppMonitorOutput,
        crate::operation::create_app_monitor::CreateAppMonitorError,
    > for CreateAppMonitorFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_app_monitor::CreateAppMonitorOutput,
            crate::operation::create_app_monitor::CreateAppMonitorError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateAppMonitorFluentBuilder {
    /// Creates a new `CreateAppMonitorFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateAppMonitor as a reference.
    pub fn as_input(&self) -> &crate::operation::create_app_monitor::builders::CreateAppMonitorInputBuilder {
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
        crate::operation::create_app_monitor::CreateAppMonitorOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_app_monitor::CreateAppMonitorError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_app_monitor::CreateAppMonitor::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_app_monitor::CreateAppMonitor::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_app_monitor::CreateAppMonitorOutput,
        crate::operation::create_app_monitor::CreateAppMonitorError,
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
    /// <p>A name for the app monitor.</p>
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.name(input.into());
        self
    }
    /// <p>A name for the app monitor.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_name(input);
        self
    }
    /// <p>A name for the app monitor.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_name()
    }
    /// <p>The top-level internet domain name for which your application has administrative authority.</p>
    pub fn domain(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.domain(input.into());
        self
    }
    /// <p>The top-level internet domain name for which your application has administrative authority.</p>
    pub fn set_domain(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_domain(input);
        self
    }
    /// <p>The top-level internet domain name for which your application has administrative authority.</p>
    pub fn get_domain(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_domain()
    }
    ///
    /// Appends an item to `DomainList`.
    ///
    /// To override the contents of this collection use [`set_domain_list`](Self::set_domain_list).
    ///
    /// <p>List the domain names for which your application has administrative authority. The <code>CreateAppMonitor</code> requires either the domain or the domain list.</p>
    pub fn domain_list(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.domain_list(input.into());
        self
    }
    /// <p>List the domain names for which your application has administrative authority. The <code>CreateAppMonitor</code> requires either the domain or the domain list.</p>
    pub fn set_domain_list(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_domain_list(input);
        self
    }
    /// <p>List the domain names for which your application has administrative authority. The <code>CreateAppMonitor</code> requires either the domain or the domain list.</p>
    pub fn get_domain_list(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_domain_list()
    }
    ///
    /// Adds a key-value pair to `Tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>Assigns one or more tags (key-value pairs) to the app monitor.</p>
    /// <p>Tags can help you organize and categorize your resources. You can also use them to scope user permissions by granting a user permission to access or change only resources with certain tag values.</p>
    /// <p>Tags don't have any semantic meaning to Amazon Web Services and are interpreted strictly as strings of characters.</p>
    /// <p>You can associate as many as 50 tags with an app monitor.</p>
    /// <p>For more information, see <a href="https://docs.aws.amazon.com/general/latest/gr/aws_tagging.html">Tagging Amazon Web Services resources</a>.</p>
    pub fn tags(mut self, k: impl ::std::convert::Into<::std::string::String>, v: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.tags(k.into(), v.into());
        self
    }
    /// <p>Assigns one or more tags (key-value pairs) to the app monitor.</p>
    /// <p>Tags can help you organize and categorize your resources. You can also use them to scope user permissions by granting a user permission to access or change only resources with certain tag values.</p>
    /// <p>Tags don't have any semantic meaning to Amazon Web Services and are interpreted strictly as strings of characters.</p>
    /// <p>You can associate as many as 50 tags with an app monitor.</p>
    /// <p>For more information, see <a href="https://docs.aws.amazon.com/general/latest/gr/aws_tagging.html">Tagging Amazon Web Services resources</a>.</p>
    pub fn set_tags(mut self, input: ::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>Assigns one or more tags (key-value pairs) to the app monitor.</p>
    /// <p>Tags can help you organize and categorize your resources. You can also use them to scope user permissions by granting a user permission to access or change only resources with certain tag values.</p>
    /// <p>Tags don't have any semantic meaning to Amazon Web Services and are interpreted strictly as strings of characters.</p>
    /// <p>You can associate as many as 50 tags with an app monitor.</p>
    /// <p>For more information, see <a href="https://docs.aws.amazon.com/general/latest/gr/aws_tagging.html">Tagging Amazon Web Services resources</a>.</p>
    pub fn get_tags(&self) -> &::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>> {
        self.inner.get_tags()
    }
    /// <p>A structure that contains much of the configuration data for the app monitor. If you are using Amazon Cognito for authorization, you must include this structure in your request, and it must include the ID of the Amazon Cognito identity pool to use for authorization. If you don't include <code>AppMonitorConfiguration</code>, you must set up your own authorization method. For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-RUM-get-started-authorization.html">Authorize your application to send data to Amazon Web Services</a>.</p>
    /// <p>If you omit this argument, the sample rate used for RUM is set to 10% of the user sessions.</p>
    pub fn app_monitor_configuration(mut self, input: crate::types::AppMonitorConfiguration) -> Self {
        self.inner = self.inner.app_monitor_configuration(input);
        self
    }
    /// <p>A structure that contains much of the configuration data for the app monitor. If you are using Amazon Cognito for authorization, you must include this structure in your request, and it must include the ID of the Amazon Cognito identity pool to use for authorization. If you don't include <code>AppMonitorConfiguration</code>, you must set up your own authorization method. For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-RUM-get-started-authorization.html">Authorize your application to send data to Amazon Web Services</a>.</p>
    /// <p>If you omit this argument, the sample rate used for RUM is set to 10% of the user sessions.</p>
    pub fn set_app_monitor_configuration(mut self, input: ::std::option::Option<crate::types::AppMonitorConfiguration>) -> Self {
        self.inner = self.inner.set_app_monitor_configuration(input);
        self
    }
    /// <p>A structure that contains much of the configuration data for the app monitor. If you are using Amazon Cognito for authorization, you must include this structure in your request, and it must include the ID of the Amazon Cognito identity pool to use for authorization. If you don't include <code>AppMonitorConfiguration</code>, you must set up your own authorization method. For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-RUM-get-started-authorization.html">Authorize your application to send data to Amazon Web Services</a>.</p>
    /// <p>If you omit this argument, the sample rate used for RUM is set to 10% of the user sessions.</p>
    pub fn get_app_monitor_configuration(&self) -> &::std::option::Option<crate::types::AppMonitorConfiguration> {
        self.inner.get_app_monitor_configuration()
    }
    /// <p>Data collected by RUM is kept by RUM for 30 days and then deleted. This parameter specifies whether RUM sends a copy of this telemetry data to Amazon CloudWatch Logs in your account. This enables you to keep the telemetry data for more than 30 days, but it does incur Amazon CloudWatch Logs charges.</p>
    /// <p>If you omit this parameter, the default is <code>false</code>.</p>
    pub fn cw_log_enabled(mut self, input: bool) -> Self {
        self.inner = self.inner.cw_log_enabled(input);
        self
    }
    /// <p>Data collected by RUM is kept by RUM for 30 days and then deleted. This parameter specifies whether RUM sends a copy of this telemetry data to Amazon CloudWatch Logs in your account. This enables you to keep the telemetry data for more than 30 days, but it does incur Amazon CloudWatch Logs charges.</p>
    /// <p>If you omit this parameter, the default is <code>false</code>.</p>
    pub fn set_cw_log_enabled(mut self, input: ::std::option::Option<bool>) -> Self {
        self.inner = self.inner.set_cw_log_enabled(input);
        self
    }
    /// <p>Data collected by RUM is kept by RUM for 30 days and then deleted. This parameter specifies whether RUM sends a copy of this telemetry data to Amazon CloudWatch Logs in your account. This enables you to keep the telemetry data for more than 30 days, but it does incur Amazon CloudWatch Logs charges.</p>
    /// <p>If you omit this parameter, the default is <code>false</code>.</p>
    pub fn get_cw_log_enabled(&self) -> &::std::option::Option<bool> {
        self.inner.get_cw_log_enabled()
    }
    /// <p>Specifies whether this app monitor allows the web client to define and send custom events. If you omit this parameter, custom events are <code>DISABLED</code>.</p>
    /// <p>For more information about custom events, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-RUM-custom-events.html">Send custom events</a>.</p>
    pub fn custom_events(mut self, input: crate::types::CustomEvents) -> Self {
        self.inner = self.inner.custom_events(input);
        self
    }
    /// <p>Specifies whether this app monitor allows the web client to define and send custom events. If you omit this parameter, custom events are <code>DISABLED</code>.</p>
    /// <p>For more information about custom events, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-RUM-custom-events.html">Send custom events</a>.</p>
    pub fn set_custom_events(mut self, input: ::std::option::Option<crate::types::CustomEvents>) -> Self {
        self.inner = self.inner.set_custom_events(input);
        self
    }
    /// <p>Specifies whether this app monitor allows the web client to define and send custom events. If you omit this parameter, custom events are <code>DISABLED</code>.</p>
    /// <p>For more information about custom events, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-RUM-custom-events.html">Send custom events</a>.</p>
    pub fn get_custom_events(&self) -> &::std::option::Option<crate::types::CustomEvents> {
        self.inner.get_custom_events()
    }
    /// <p>A structure that contains the configuration for how an app monitor can deobfuscate stack traces.</p>
    pub fn deobfuscation_configuration(mut self, input: crate::types::DeobfuscationConfiguration) -> Self {
        self.inner = self.inner.deobfuscation_configuration(input);
        self
    }
    /// <p>A structure that contains the configuration for how an app monitor can deobfuscate stack traces.</p>
    pub fn set_deobfuscation_configuration(mut self, input: ::std::option::Option<crate::types::DeobfuscationConfiguration>) -> Self {
        self.inner = self.inner.set_deobfuscation_configuration(input);
        self
    }
    /// <p>A structure that contains the configuration for how an app monitor can deobfuscate stack traces.</p>
    pub fn get_deobfuscation_configuration(&self) -> &::std::option::Option<crate::types::DeobfuscationConfiguration> {
        self.inner.get_deobfuscation_configuration()
    }
}
