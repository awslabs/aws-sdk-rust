// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::batch_create_bill_scenario_usage_modification::_batch_create_bill_scenario_usage_modification_output::BatchCreateBillScenarioUsageModificationOutputBuilder;

pub use crate::operation::batch_create_bill_scenario_usage_modification::_batch_create_bill_scenario_usage_modification_input::BatchCreateBillScenarioUsageModificationInputBuilder;

impl crate::operation::batch_create_bill_scenario_usage_modification::builders::BatchCreateBillScenarioUsageModificationInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModificationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModificationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.batch_create_bill_scenario_usage_modification();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `BatchCreateBillScenarioUsageModification`.
///
/// <p>Create Amazon Web Services service usage that you want to model in a Bill Scenario.</p><note>
/// <p>The <code>BatchCreateBillScenarioUsageModification</code> operation doesn't have its own IAM permission. To authorize this operation for Amazon Web Services principals, include the permission <code>bcm-pricing-calculator:CreateBillScenarioUsageModification</code> in your policies.</p>
/// </note>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct BatchCreateBillScenarioUsageModificationFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::batch_create_bill_scenario_usage_modification::builders::BatchCreateBillScenarioUsageModificationInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModificationOutput,
        crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModificationError,
    > for BatchCreateBillScenarioUsageModificationFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModificationOutput,
            crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModificationError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl BatchCreateBillScenarioUsageModificationFluentBuilder {
    /// Creates a new `BatchCreateBillScenarioUsageModificationFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the BatchCreateBillScenarioUsageModification as a reference.
    pub fn as_input(
        &self,
    ) -> &crate::operation::batch_create_bill_scenario_usage_modification::builders::BatchCreateBillScenarioUsageModificationInputBuilder {
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
        crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModificationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModificationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins =
            crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModification::operation_runtime_plugins(
                self.handle.runtime_plugins.clone(),
                &self.handle.conf,
                self.config_override,
            );
        crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModification::orchestrate(
            &runtime_plugins,
            input,
        )
        .await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModificationOutput,
        crate::operation::batch_create_bill_scenario_usage_modification::BatchCreateBillScenarioUsageModificationError,
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
    /// <p>The ID of the Bill Scenario for which you want to create the modeled usage.</p>
    pub fn bill_scenario_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.bill_scenario_id(input.into());
        self
    }
    /// <p>The ID of the Bill Scenario for which you want to create the modeled usage.</p>
    pub fn set_bill_scenario_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_bill_scenario_id(input);
        self
    }
    /// <p>The ID of the Bill Scenario for which you want to create the modeled usage.</p>
    pub fn get_bill_scenario_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_bill_scenario_id()
    }
    ///
    /// Appends an item to `usageModifications`.
    ///
    /// To override the contents of this collection use [`set_usage_modifications`](Self::set_usage_modifications).
    ///
    /// <p>List of usage that you want to model in the Bill Scenario.</p>
    pub fn usage_modifications(mut self, input: crate::types::BatchCreateBillScenarioUsageModificationEntry) -> Self {
        self.inner = self.inner.usage_modifications(input);
        self
    }
    /// <p>List of usage that you want to model in the Bill Scenario.</p>
    pub fn set_usage_modifications(
        mut self,
        input: ::std::option::Option<::std::vec::Vec<crate::types::BatchCreateBillScenarioUsageModificationEntry>>,
    ) -> Self {
        self.inner = self.inner.set_usage_modifications(input);
        self
    }
    /// <p>List of usage that you want to model in the Bill Scenario.</p>
    pub fn get_usage_modifications(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::BatchCreateBillScenarioUsageModificationEntry>> {
        self.inner.get_usage_modifications()
    }
    /// <p>A unique, case-sensitive identifier that you provide to ensure the idempotency of the request.</p>
    pub fn client_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_token(input.into());
        self
    }
    /// <p>A unique, case-sensitive identifier that you provide to ensure the idempotency of the request.</p>
    pub fn set_client_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_token(input);
        self
    }
    /// <p>A unique, case-sensitive identifier that you provide to ensure the idempotency of the request.</p>
    pub fn get_client_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_token()
    }
}
