// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::list_license_configurations::_list_license_configurations_output::ListLicenseConfigurationsOutputBuilder;

pub use crate::operation::list_license_configurations::_list_license_configurations_input::ListLicenseConfigurationsInputBuilder;

impl crate::operation::list_license_configurations::builders::ListLicenseConfigurationsInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::list_license_configurations::ListLicenseConfigurationsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::list_license_configurations::ListLicenseConfigurationsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.list_license_configurations();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `ListLicenseConfigurations`.
///
/// <p>Lists the license configurations for your account.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct ListLicenseConfigurationsFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::list_license_configurations::builders::ListLicenseConfigurationsInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::list_license_configurations::ListLicenseConfigurationsOutput,
        crate::operation::list_license_configurations::ListLicenseConfigurationsError,
    > for ListLicenseConfigurationsFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::list_license_configurations::ListLicenseConfigurationsOutput,
            crate::operation::list_license_configurations::ListLicenseConfigurationsError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl ListLicenseConfigurationsFluentBuilder {
    /// Creates a new `ListLicenseConfigurationsFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the ListLicenseConfigurations as a reference.
    pub fn as_input(&self) -> &crate::operation::list_license_configurations::builders::ListLicenseConfigurationsInputBuilder {
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
        crate::operation::list_license_configurations::ListLicenseConfigurationsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::list_license_configurations::ListLicenseConfigurationsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::list_license_configurations::ListLicenseConfigurations::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::list_license_configurations::ListLicenseConfigurations::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::list_license_configurations::ListLicenseConfigurationsOutput,
        crate::operation::list_license_configurations::ListLicenseConfigurationsError,
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
    ///
    /// Appends an item to `LicenseConfigurationArns`.
    ///
    /// To override the contents of this collection use [`set_license_configuration_arns`](Self::set_license_configuration_arns).
    ///
    /// <p>Amazon Resource Names (ARN) of the license configurations.</p>
    pub fn license_configuration_arns(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.license_configuration_arns(input.into());
        self
    }
    /// <p>Amazon Resource Names (ARN) of the license configurations.</p>
    pub fn set_license_configuration_arns(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_license_configuration_arns(input);
        self
    }
    /// <p>Amazon Resource Names (ARN) of the license configurations.</p>
    pub fn get_license_configuration_arns(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_license_configuration_arns()
    }
    /// <p>Maximum number of results to return in a single call.</p>
    pub fn max_results(mut self, input: i32) -> Self {
        self.inner = self.inner.max_results(input);
        self
    }
    /// <p>Maximum number of results to return in a single call.</p>
    pub fn set_max_results(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_max_results(input);
        self
    }
    /// <p>Maximum number of results to return in a single call.</p>
    pub fn get_max_results(&self) -> &::std::option::Option<i32> {
        self.inner.get_max_results()
    }
    /// <p>Token for the next set of results.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.next_token(input.into());
        self
    }
    /// <p>Token for the next set of results.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_next_token(input);
        self
    }
    /// <p>Token for the next set of results.</p>
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_next_token()
    }
    ///
    /// Appends an item to `Filters`.
    ///
    /// To override the contents of this collection use [`set_filters`](Self::set_filters).
    ///
    /// <p>Filters to scope the results. The following filters and logical operators are supported:</p>
    /// <ul>
    /// <li>
    /// <p><code>licenseCountingType</code> - The dimension for which licenses are counted. Possible values are <code>vCPU</code> | <code>Instance</code> | <code>Core</code> | <code>Socket</code>.</p></li>
    /// <li>
    /// <p><code>enforceLicenseCount</code> - A Boolean value that indicates whether hard license enforcement is used.</p></li>
    /// <li>
    /// <p><code>usagelimitExceeded</code> - A Boolean value that indicates whether the available licenses have been exceeded.</p></li>
    /// </ul>
    pub fn filters(mut self, input: crate::types::Filter) -> Self {
        self.inner = self.inner.filters(input);
        self
    }
    /// <p>Filters to scope the results. The following filters and logical operators are supported:</p>
    /// <ul>
    /// <li>
    /// <p><code>licenseCountingType</code> - The dimension for which licenses are counted. Possible values are <code>vCPU</code> | <code>Instance</code> | <code>Core</code> | <code>Socket</code>.</p></li>
    /// <li>
    /// <p><code>enforceLicenseCount</code> - A Boolean value that indicates whether hard license enforcement is used.</p></li>
    /// <li>
    /// <p><code>usagelimitExceeded</code> - A Boolean value that indicates whether the available licenses have been exceeded.</p></li>
    /// </ul>
    pub fn set_filters(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Filter>>) -> Self {
        self.inner = self.inner.set_filters(input);
        self
    }
    /// <p>Filters to scope the results. The following filters and logical operators are supported:</p>
    /// <ul>
    /// <li>
    /// <p><code>licenseCountingType</code> - The dimension for which licenses are counted. Possible values are <code>vCPU</code> | <code>Instance</code> | <code>Core</code> | <code>Socket</code>.</p></li>
    /// <li>
    /// <p><code>enforceLicenseCount</code> - A Boolean value that indicates whether hard license enforcement is used.</p></li>
    /// <li>
    /// <p><code>usagelimitExceeded</code> - A Boolean value that indicates whether the available licenses have been exceeded.</p></li>
    /// </ul>
    pub fn get_filters(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Filter>> {
        self.inner.get_filters()
    }
}
