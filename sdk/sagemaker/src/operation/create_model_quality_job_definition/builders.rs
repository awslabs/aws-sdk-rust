// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_model_quality_job_definition::_create_model_quality_job_definition_output::CreateModelQualityJobDefinitionOutputBuilder;

pub use crate::operation::create_model_quality_job_definition::_create_model_quality_job_definition_input::CreateModelQualityJobDefinitionInputBuilder;

impl crate::operation::create_model_quality_job_definition::builders::CreateModelQualityJobDefinitionInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinitionOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinitionError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_model_quality_job_definition();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateModelQualityJobDefinition`.
///
/// <p>Creates a definition for a job that monitors model quality and drift. For information about model monitor, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/model-monitor.html">Amazon SageMaker AI Model Monitor</a>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateModelQualityJobDefinitionFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_model_quality_job_definition::builders::CreateModelQualityJobDefinitionInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinitionOutput,
        crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinitionError,
    > for CreateModelQualityJobDefinitionFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinitionOutput,
            crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinitionError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateModelQualityJobDefinitionFluentBuilder {
    /// Creates a new `CreateModelQualityJobDefinitionFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateModelQualityJobDefinition as a reference.
    pub fn as_input(&self) -> &crate::operation::create_model_quality_job_definition::builders::CreateModelQualityJobDefinitionInputBuilder {
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
        crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinitionOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinitionError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinition::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinition::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinitionOutput,
        crate::operation::create_model_quality_job_definition::CreateModelQualityJobDefinitionError,
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
    /// <p>The name of the monitoring job definition.</p>
    pub fn job_definition_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.job_definition_name(input.into());
        self
    }
    /// <p>The name of the monitoring job definition.</p>
    pub fn set_job_definition_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_job_definition_name(input);
        self
    }
    /// <p>The name of the monitoring job definition.</p>
    pub fn get_job_definition_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_job_definition_name()
    }
    /// <p>Specifies the constraints and baselines for the monitoring job.</p>
    pub fn model_quality_baseline_config(mut self, input: crate::types::ModelQualityBaselineConfig) -> Self {
        self.inner = self.inner.model_quality_baseline_config(input);
        self
    }
    /// <p>Specifies the constraints and baselines for the monitoring job.</p>
    pub fn set_model_quality_baseline_config(mut self, input: ::std::option::Option<crate::types::ModelQualityBaselineConfig>) -> Self {
        self.inner = self.inner.set_model_quality_baseline_config(input);
        self
    }
    /// <p>Specifies the constraints and baselines for the monitoring job.</p>
    pub fn get_model_quality_baseline_config(&self) -> &::std::option::Option<crate::types::ModelQualityBaselineConfig> {
        self.inner.get_model_quality_baseline_config()
    }
    /// <p>The container that runs the monitoring job.</p>
    pub fn model_quality_app_specification(mut self, input: crate::types::ModelQualityAppSpecification) -> Self {
        self.inner = self.inner.model_quality_app_specification(input);
        self
    }
    /// <p>The container that runs the monitoring job.</p>
    pub fn set_model_quality_app_specification(mut self, input: ::std::option::Option<crate::types::ModelQualityAppSpecification>) -> Self {
        self.inner = self.inner.set_model_quality_app_specification(input);
        self
    }
    /// <p>The container that runs the monitoring job.</p>
    pub fn get_model_quality_app_specification(&self) -> &::std::option::Option<crate::types::ModelQualityAppSpecification> {
        self.inner.get_model_quality_app_specification()
    }
    /// <p>A list of the inputs that are monitored. Currently endpoints are supported.</p>
    pub fn model_quality_job_input(mut self, input: crate::types::ModelQualityJobInput) -> Self {
        self.inner = self.inner.model_quality_job_input(input);
        self
    }
    /// <p>A list of the inputs that are monitored. Currently endpoints are supported.</p>
    pub fn set_model_quality_job_input(mut self, input: ::std::option::Option<crate::types::ModelQualityJobInput>) -> Self {
        self.inner = self.inner.set_model_quality_job_input(input);
        self
    }
    /// <p>A list of the inputs that are monitored. Currently endpoints are supported.</p>
    pub fn get_model_quality_job_input(&self) -> &::std::option::Option<crate::types::ModelQualityJobInput> {
        self.inner.get_model_quality_job_input()
    }
    /// <p>The output configuration for monitoring jobs.</p>
    pub fn model_quality_job_output_config(mut self, input: crate::types::MonitoringOutputConfig) -> Self {
        self.inner = self.inner.model_quality_job_output_config(input);
        self
    }
    /// <p>The output configuration for monitoring jobs.</p>
    pub fn set_model_quality_job_output_config(mut self, input: ::std::option::Option<crate::types::MonitoringOutputConfig>) -> Self {
        self.inner = self.inner.set_model_quality_job_output_config(input);
        self
    }
    /// <p>The output configuration for monitoring jobs.</p>
    pub fn get_model_quality_job_output_config(&self) -> &::std::option::Option<crate::types::MonitoringOutputConfig> {
        self.inner.get_model_quality_job_output_config()
    }
    /// <p>Identifies the resources to deploy for a monitoring job.</p>
    pub fn job_resources(mut self, input: crate::types::MonitoringResources) -> Self {
        self.inner = self.inner.job_resources(input);
        self
    }
    /// <p>Identifies the resources to deploy for a monitoring job.</p>
    pub fn set_job_resources(mut self, input: ::std::option::Option<crate::types::MonitoringResources>) -> Self {
        self.inner = self.inner.set_job_resources(input);
        self
    }
    /// <p>Identifies the resources to deploy for a monitoring job.</p>
    pub fn get_job_resources(&self) -> &::std::option::Option<crate::types::MonitoringResources> {
        self.inner.get_job_resources()
    }
    /// <p>Specifies the network configuration for the monitoring job.</p>
    pub fn network_config(mut self, input: crate::types::MonitoringNetworkConfig) -> Self {
        self.inner = self.inner.network_config(input);
        self
    }
    /// <p>Specifies the network configuration for the monitoring job.</p>
    pub fn set_network_config(mut self, input: ::std::option::Option<crate::types::MonitoringNetworkConfig>) -> Self {
        self.inner = self.inner.set_network_config(input);
        self
    }
    /// <p>Specifies the network configuration for the monitoring job.</p>
    pub fn get_network_config(&self) -> &::std::option::Option<crate::types::MonitoringNetworkConfig> {
        self.inner.get_network_config()
    }
    /// <p>The Amazon Resource Name (ARN) of an IAM role that Amazon SageMaker AI can assume to perform tasks on your behalf.</p>
    pub fn role_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.role_arn(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of an IAM role that Amazon SageMaker AI can assume to perform tasks on your behalf.</p>
    pub fn set_role_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_role_arn(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) of an IAM role that Amazon SageMaker AI can assume to perform tasks on your behalf.</p>
    pub fn get_role_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_role_arn()
    }
    /// <p>A time limit for how long the monitoring job is allowed to run before stopping.</p>
    pub fn stopping_condition(mut self, input: crate::types::MonitoringStoppingCondition) -> Self {
        self.inner = self.inner.stopping_condition(input);
        self
    }
    /// <p>A time limit for how long the monitoring job is allowed to run before stopping.</p>
    pub fn set_stopping_condition(mut self, input: ::std::option::Option<crate::types::MonitoringStoppingCondition>) -> Self {
        self.inner = self.inner.set_stopping_condition(input);
        self
    }
    /// <p>A time limit for how long the monitoring job is allowed to run before stopping.</p>
    pub fn get_stopping_condition(&self) -> &::std::option::Option<crate::types::MonitoringStoppingCondition> {
        self.inner.get_stopping_condition()
    }
    ///
    /// Appends an item to `Tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>(Optional) An array of key-value pairs. For more information, see <a href="https://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/cost-alloc-tags.html#allocation-whatURL"> Using Cost Allocation Tags</a> in the <i>Amazon Web Services Billing and Cost Management User Guide</i>.</p>
    pub fn tags(mut self, input: crate::types::Tag) -> Self {
        self.inner = self.inner.tags(input);
        self
    }
    /// <p>(Optional) An array of key-value pairs. For more information, see <a href="https://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/cost-alloc-tags.html#allocation-whatURL"> Using Cost Allocation Tags</a> in the <i>Amazon Web Services Billing and Cost Management User Guide</i>.</p>
    pub fn set_tags(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Tag>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>(Optional) An array of key-value pairs. For more information, see <a href="https://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/cost-alloc-tags.html#allocation-whatURL"> Using Cost Allocation Tags</a> in the <i>Amazon Web Services Billing and Cost Management User Guide</i>.</p>
    pub fn get_tags(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Tag>> {
        self.inner.get_tags()
    }
}
