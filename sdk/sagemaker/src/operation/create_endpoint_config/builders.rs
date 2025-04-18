// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_endpoint_config::_create_endpoint_config_output::CreateEndpointConfigOutputBuilder;

pub use crate::operation::create_endpoint_config::_create_endpoint_config_input::CreateEndpointConfigInputBuilder;

impl crate::operation::create_endpoint_config::builders::CreateEndpointConfigInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_endpoint_config::CreateEndpointConfigOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_endpoint_config::CreateEndpointConfigError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_endpoint_config();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateEndpointConfig`.
///
/// <p>Creates an endpoint configuration that SageMaker hosting services uses to deploy models. In the configuration, you identify one or more models, created using the <code>CreateModel</code> API, to deploy and the resources that you want SageMaker to provision. Then you call the <a href="https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_CreateEndpoint.html">CreateEndpoint</a> API.</p><note>
/// <p>Use this API if you want to use SageMaker hosting services to deploy models into production.</p>
/// </note>
/// <p>In the request, you define a <code>ProductionVariant</code>, for each model that you want to deploy. Each <code>ProductionVariant</code> parameter also describes the resources that you want SageMaker to provision. This includes the number and type of ML compute instances to deploy.</p>
/// <p>If you are hosting multiple models, you also assign a <code>VariantWeight</code> to specify how much traffic you want to allocate to each model. For example, suppose that you want to host two models, A and B, and you assign traffic weight 2 for model A and 1 for model B. SageMaker distributes two-thirds of the traffic to Model A, and one-third to model B.</p><note>
/// <p>When you call <a href="https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_CreateEndpoint.html">CreateEndpoint</a>, a load call is made to DynamoDB to verify that your endpoint configuration exists. When you read data from a DynamoDB table supporting <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.ReadConsistency.html"> <code>Eventually Consistent Reads</code> </a>, the response might not reflect the results of a recently completed write operation. The response might include some stale data. If the dependent entities are not yet in DynamoDB, this causes a validation error. If you repeat your read request after a short time, the response should return the latest data. So retry logic is recommended to handle these possible issues. We also recommend that customers call <a href="https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_DescribeEndpointConfig.html">DescribeEndpointConfig</a> before calling <a href="https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_CreateEndpoint.html">CreateEndpoint</a> to minimize the potential impact of a DynamoDB eventually consistent read.</p>
/// </note>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateEndpointConfigFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_endpoint_config::builders::CreateEndpointConfigInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_endpoint_config::CreateEndpointConfigOutput,
        crate::operation::create_endpoint_config::CreateEndpointConfigError,
    > for CreateEndpointConfigFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_endpoint_config::CreateEndpointConfigOutput,
            crate::operation::create_endpoint_config::CreateEndpointConfigError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateEndpointConfigFluentBuilder {
    /// Creates a new `CreateEndpointConfigFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateEndpointConfig as a reference.
    pub fn as_input(&self) -> &crate::operation::create_endpoint_config::builders::CreateEndpointConfigInputBuilder {
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
        crate::operation::create_endpoint_config::CreateEndpointConfigOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_endpoint_config::CreateEndpointConfigError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_endpoint_config::CreateEndpointConfig::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_endpoint_config::CreateEndpointConfig::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_endpoint_config::CreateEndpointConfigOutput,
        crate::operation::create_endpoint_config::CreateEndpointConfigError,
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
    /// <p>The name of the endpoint configuration. You specify this name in a <a href="https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_CreateEndpoint.html">CreateEndpoint</a> request.</p>
    pub fn endpoint_config_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.endpoint_config_name(input.into());
        self
    }
    /// <p>The name of the endpoint configuration. You specify this name in a <a href="https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_CreateEndpoint.html">CreateEndpoint</a> request.</p>
    pub fn set_endpoint_config_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_endpoint_config_name(input);
        self
    }
    /// <p>The name of the endpoint configuration. You specify this name in a <a href="https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_CreateEndpoint.html">CreateEndpoint</a> request.</p>
    pub fn get_endpoint_config_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_endpoint_config_name()
    }
    ///
    /// Appends an item to `ProductionVariants`.
    ///
    /// To override the contents of this collection use [`set_production_variants`](Self::set_production_variants).
    ///
    /// <p>An array of <code>ProductionVariant</code> objects, one for each model that you want to host at this endpoint.</p>
    pub fn production_variants(mut self, input: crate::types::ProductionVariant) -> Self {
        self.inner = self.inner.production_variants(input);
        self
    }
    /// <p>An array of <code>ProductionVariant</code> objects, one for each model that you want to host at this endpoint.</p>
    pub fn set_production_variants(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::ProductionVariant>>) -> Self {
        self.inner = self.inner.set_production_variants(input);
        self
    }
    /// <p>An array of <code>ProductionVariant</code> objects, one for each model that you want to host at this endpoint.</p>
    pub fn get_production_variants(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::ProductionVariant>> {
        self.inner.get_production_variants()
    }
    /// <p>Configuration to control how SageMaker AI captures inference data.</p>
    pub fn data_capture_config(mut self, input: crate::types::DataCaptureConfig) -> Self {
        self.inner = self.inner.data_capture_config(input);
        self
    }
    /// <p>Configuration to control how SageMaker AI captures inference data.</p>
    pub fn set_data_capture_config(mut self, input: ::std::option::Option<crate::types::DataCaptureConfig>) -> Self {
        self.inner = self.inner.set_data_capture_config(input);
        self
    }
    /// <p>Configuration to control how SageMaker AI captures inference data.</p>
    pub fn get_data_capture_config(&self) -> &::std::option::Option<crate::types::DataCaptureConfig> {
        self.inner.get_data_capture_config()
    }
    ///
    /// Appends an item to `Tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>An array of key-value pairs. You can use tags to categorize your Amazon Web Services resources in different ways, for example, by purpose, owner, or environment. For more information, see <a href="https://docs.aws.amazon.com/general/latest/gr/aws_tagging.html">Tagging Amazon Web Services Resources</a>.</p>
    pub fn tags(mut self, input: crate::types::Tag) -> Self {
        self.inner = self.inner.tags(input);
        self
    }
    /// <p>An array of key-value pairs. You can use tags to categorize your Amazon Web Services resources in different ways, for example, by purpose, owner, or environment. For more information, see <a href="https://docs.aws.amazon.com/general/latest/gr/aws_tagging.html">Tagging Amazon Web Services Resources</a>.</p>
    pub fn set_tags(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Tag>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>An array of key-value pairs. You can use tags to categorize your Amazon Web Services resources in different ways, for example, by purpose, owner, or environment. For more information, see <a href="https://docs.aws.amazon.com/general/latest/gr/aws_tagging.html">Tagging Amazon Web Services Resources</a>.</p>
    pub fn get_tags(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Tag>> {
        self.inner.get_tags()
    }
    /// <p>The Amazon Resource Name (ARN) of a Amazon Web Services Key Management Service key that SageMaker uses to encrypt data on the storage volume attached to the ML compute instance that hosts the endpoint.</p>
    /// <p>The KmsKeyId can be any of the following formats:</p>
    /// <ul>
    /// <li>
    /// <p>Key ID: <code>1234abcd-12ab-34cd-56ef-1234567890ab</code></p></li>
    /// <li>
    /// <p>Key ARN: <code>arn:aws:kms:us-west-2:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab</code></p></li>
    /// <li>
    /// <p>Alias name: <code>alias/ExampleAlias</code></p></li>
    /// <li>
    /// <p>Alias name ARN: <code>arn:aws:kms:us-west-2:111122223333:alias/ExampleAlias</code></p></li>
    /// </ul>
    /// <p>The KMS key policy must grant permission to the IAM role that you specify in your <code>CreateEndpoint</code>, <code>UpdateEndpoint</code> requests. For more information, refer to the Amazon Web Services Key Management Service section<a href="https://docs.aws.amazon.com/kms/latest/developerguide/key-policies.html"> Using Key Policies in Amazon Web Services KMS </a></p><note>
    /// <p>Certain Nitro-based instances include local storage, dependent on the instance type. Local storage volumes are encrypted using a hardware module on the instance. You can't request a <code>KmsKeyId</code> when using an instance type with local storage. If any of the models that you specify in the <code>ProductionVariants</code> parameter use nitro-based instances with local storage, do not specify a value for the <code>KmsKeyId</code> parameter. If you specify a value for <code>KmsKeyId</code> when using any nitro-based instances with local storage, the call to <code>CreateEndpointConfig</code> fails.</p>
    /// <p>For a list of instance types that support local instance storage, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/InstanceStorage.html#instance-store-volumes">Instance Store Volumes</a>.</p>
    /// <p>For more information about local instance storage encryption, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ssd-instance-store.html">SSD Instance Store Volumes</a>.</p>
    /// </note>
    pub fn kms_key_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.kms_key_id(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of a Amazon Web Services Key Management Service key that SageMaker uses to encrypt data on the storage volume attached to the ML compute instance that hosts the endpoint.</p>
    /// <p>The KmsKeyId can be any of the following formats:</p>
    /// <ul>
    /// <li>
    /// <p>Key ID: <code>1234abcd-12ab-34cd-56ef-1234567890ab</code></p></li>
    /// <li>
    /// <p>Key ARN: <code>arn:aws:kms:us-west-2:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab</code></p></li>
    /// <li>
    /// <p>Alias name: <code>alias/ExampleAlias</code></p></li>
    /// <li>
    /// <p>Alias name ARN: <code>arn:aws:kms:us-west-2:111122223333:alias/ExampleAlias</code></p></li>
    /// </ul>
    /// <p>The KMS key policy must grant permission to the IAM role that you specify in your <code>CreateEndpoint</code>, <code>UpdateEndpoint</code> requests. For more information, refer to the Amazon Web Services Key Management Service section<a href="https://docs.aws.amazon.com/kms/latest/developerguide/key-policies.html"> Using Key Policies in Amazon Web Services KMS </a></p><note>
    /// <p>Certain Nitro-based instances include local storage, dependent on the instance type. Local storage volumes are encrypted using a hardware module on the instance. You can't request a <code>KmsKeyId</code> when using an instance type with local storage. If any of the models that you specify in the <code>ProductionVariants</code> parameter use nitro-based instances with local storage, do not specify a value for the <code>KmsKeyId</code> parameter. If you specify a value for <code>KmsKeyId</code> when using any nitro-based instances with local storage, the call to <code>CreateEndpointConfig</code> fails.</p>
    /// <p>For a list of instance types that support local instance storage, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/InstanceStorage.html#instance-store-volumes">Instance Store Volumes</a>.</p>
    /// <p>For more information about local instance storage encryption, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ssd-instance-store.html">SSD Instance Store Volumes</a>.</p>
    /// </note>
    pub fn set_kms_key_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_kms_key_id(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) of a Amazon Web Services Key Management Service key that SageMaker uses to encrypt data on the storage volume attached to the ML compute instance that hosts the endpoint.</p>
    /// <p>The KmsKeyId can be any of the following formats:</p>
    /// <ul>
    /// <li>
    /// <p>Key ID: <code>1234abcd-12ab-34cd-56ef-1234567890ab</code></p></li>
    /// <li>
    /// <p>Key ARN: <code>arn:aws:kms:us-west-2:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab</code></p></li>
    /// <li>
    /// <p>Alias name: <code>alias/ExampleAlias</code></p></li>
    /// <li>
    /// <p>Alias name ARN: <code>arn:aws:kms:us-west-2:111122223333:alias/ExampleAlias</code></p></li>
    /// </ul>
    /// <p>The KMS key policy must grant permission to the IAM role that you specify in your <code>CreateEndpoint</code>, <code>UpdateEndpoint</code> requests. For more information, refer to the Amazon Web Services Key Management Service section<a href="https://docs.aws.amazon.com/kms/latest/developerguide/key-policies.html"> Using Key Policies in Amazon Web Services KMS </a></p><note>
    /// <p>Certain Nitro-based instances include local storage, dependent on the instance type. Local storage volumes are encrypted using a hardware module on the instance. You can't request a <code>KmsKeyId</code> when using an instance type with local storage. If any of the models that you specify in the <code>ProductionVariants</code> parameter use nitro-based instances with local storage, do not specify a value for the <code>KmsKeyId</code> parameter. If you specify a value for <code>KmsKeyId</code> when using any nitro-based instances with local storage, the call to <code>CreateEndpointConfig</code> fails.</p>
    /// <p>For a list of instance types that support local instance storage, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/InstanceStorage.html#instance-store-volumes">Instance Store Volumes</a>.</p>
    /// <p>For more information about local instance storage encryption, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ssd-instance-store.html">SSD Instance Store Volumes</a>.</p>
    /// </note>
    pub fn get_kms_key_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_kms_key_id()
    }
    /// <p>Specifies configuration for how an endpoint performs asynchronous inference. This is a required field in order for your Endpoint to be invoked using <a href="https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_runtime_InvokeEndpointAsync.html">InvokeEndpointAsync</a>.</p>
    pub fn async_inference_config(mut self, input: crate::types::AsyncInferenceConfig) -> Self {
        self.inner = self.inner.async_inference_config(input);
        self
    }
    /// <p>Specifies configuration for how an endpoint performs asynchronous inference. This is a required field in order for your Endpoint to be invoked using <a href="https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_runtime_InvokeEndpointAsync.html">InvokeEndpointAsync</a>.</p>
    pub fn set_async_inference_config(mut self, input: ::std::option::Option<crate::types::AsyncInferenceConfig>) -> Self {
        self.inner = self.inner.set_async_inference_config(input);
        self
    }
    /// <p>Specifies configuration for how an endpoint performs asynchronous inference. This is a required field in order for your Endpoint to be invoked using <a href="https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_runtime_InvokeEndpointAsync.html">InvokeEndpointAsync</a>.</p>
    pub fn get_async_inference_config(&self) -> &::std::option::Option<crate::types::AsyncInferenceConfig> {
        self.inner.get_async_inference_config()
    }
    /// <p>A member of <code>CreateEndpointConfig</code> that enables explainers.</p>
    pub fn explainer_config(mut self, input: crate::types::ExplainerConfig) -> Self {
        self.inner = self.inner.explainer_config(input);
        self
    }
    /// <p>A member of <code>CreateEndpointConfig</code> that enables explainers.</p>
    pub fn set_explainer_config(mut self, input: ::std::option::Option<crate::types::ExplainerConfig>) -> Self {
        self.inner = self.inner.set_explainer_config(input);
        self
    }
    /// <p>A member of <code>CreateEndpointConfig</code> that enables explainers.</p>
    pub fn get_explainer_config(&self) -> &::std::option::Option<crate::types::ExplainerConfig> {
        self.inner.get_explainer_config()
    }
    ///
    /// Appends an item to `ShadowProductionVariants`.
    ///
    /// To override the contents of this collection use [`set_shadow_production_variants`](Self::set_shadow_production_variants).
    ///
    /// <p>An array of <code>ProductionVariant</code> objects, one for each model that you want to host at this endpoint in shadow mode with production traffic replicated from the model specified on <code>ProductionVariants</code>. If you use this field, you can only specify one variant for <code>ProductionVariants</code> and one variant for <code>ShadowProductionVariants</code>.</p>
    pub fn shadow_production_variants(mut self, input: crate::types::ProductionVariant) -> Self {
        self.inner = self.inner.shadow_production_variants(input);
        self
    }
    /// <p>An array of <code>ProductionVariant</code> objects, one for each model that you want to host at this endpoint in shadow mode with production traffic replicated from the model specified on <code>ProductionVariants</code>. If you use this field, you can only specify one variant for <code>ProductionVariants</code> and one variant for <code>ShadowProductionVariants</code>.</p>
    pub fn set_shadow_production_variants(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::ProductionVariant>>) -> Self {
        self.inner = self.inner.set_shadow_production_variants(input);
        self
    }
    /// <p>An array of <code>ProductionVariant</code> objects, one for each model that you want to host at this endpoint in shadow mode with production traffic replicated from the model specified on <code>ProductionVariants</code>. If you use this field, you can only specify one variant for <code>ProductionVariants</code> and one variant for <code>ShadowProductionVariants</code>.</p>
    pub fn get_shadow_production_variants(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::ProductionVariant>> {
        self.inner.get_shadow_production_variants()
    }
    /// <p>The Amazon Resource Name (ARN) of an IAM role that Amazon SageMaker AI can assume to perform actions on your behalf. For more information, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/sagemaker-roles.html">SageMaker AI Roles</a>.</p><note>
    /// <p>To be able to pass this role to Amazon SageMaker AI, the caller of this action must have the <code>iam:PassRole</code> permission.</p>
    /// </note>
    pub fn execution_role_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.execution_role_arn(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of an IAM role that Amazon SageMaker AI can assume to perform actions on your behalf. For more information, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/sagemaker-roles.html">SageMaker AI Roles</a>.</p><note>
    /// <p>To be able to pass this role to Amazon SageMaker AI, the caller of this action must have the <code>iam:PassRole</code> permission.</p>
    /// </note>
    pub fn set_execution_role_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_execution_role_arn(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) of an IAM role that Amazon SageMaker AI can assume to perform actions on your behalf. For more information, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/sagemaker-roles.html">SageMaker AI Roles</a>.</p><note>
    /// <p>To be able to pass this role to Amazon SageMaker AI, the caller of this action must have the <code>iam:PassRole</code> permission.</p>
    /// </note>
    pub fn get_execution_role_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_execution_role_arn()
    }
    /// <p>Specifies an Amazon Virtual Private Cloud (VPC) that your SageMaker jobs, hosted models, and compute resources have access to. You can control access to and from your resources by configuring a VPC. For more information, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/infrastructure-give-access.html">Give SageMaker Access to Resources in your Amazon VPC</a>.</p>
    pub fn vpc_config(mut self, input: crate::types::VpcConfig) -> Self {
        self.inner = self.inner.vpc_config(input);
        self
    }
    /// <p>Specifies an Amazon Virtual Private Cloud (VPC) that your SageMaker jobs, hosted models, and compute resources have access to. You can control access to and from your resources by configuring a VPC. For more information, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/infrastructure-give-access.html">Give SageMaker Access to Resources in your Amazon VPC</a>.</p>
    pub fn set_vpc_config(mut self, input: ::std::option::Option<crate::types::VpcConfig>) -> Self {
        self.inner = self.inner.set_vpc_config(input);
        self
    }
    /// <p>Specifies an Amazon Virtual Private Cloud (VPC) that your SageMaker jobs, hosted models, and compute resources have access to. You can control access to and from your resources by configuring a VPC. For more information, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/infrastructure-give-access.html">Give SageMaker Access to Resources in your Amazon VPC</a>.</p>
    pub fn get_vpc_config(&self) -> &::std::option::Option<crate::types::VpcConfig> {
        self.inner.get_vpc_config()
    }
    /// <p>Sets whether all model containers deployed to the endpoint are isolated. If they are, no inbound or outbound network calls can be made to or from the model containers.</p>
    pub fn enable_network_isolation(mut self, input: bool) -> Self {
        self.inner = self.inner.enable_network_isolation(input);
        self
    }
    /// <p>Sets whether all model containers deployed to the endpoint are isolated. If they are, no inbound or outbound network calls can be made to or from the model containers.</p>
    pub fn set_enable_network_isolation(mut self, input: ::std::option::Option<bool>) -> Self {
        self.inner = self.inner.set_enable_network_isolation(input);
        self
    }
    /// <p>Sets whether all model containers deployed to the endpoint are isolated. If they are, no inbound or outbound network calls can be made to or from the model containers.</p>
    pub fn get_enable_network_isolation(&self) -> &::std::option::Option<bool> {
        self.inner.get_enable_network_isolation()
    }
}
