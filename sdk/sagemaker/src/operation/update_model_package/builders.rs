// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::update_model_package::_update_model_package_output::UpdateModelPackageOutputBuilder;

pub use crate::operation::update_model_package::_update_model_package_input::UpdateModelPackageInputBuilder;

impl crate::operation::update_model_package::builders::UpdateModelPackageInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::update_model_package::UpdateModelPackageOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_model_package::UpdateModelPackageError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.update_model_package();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UpdateModelPackage`.
///
/// <p>Updates a versioned model.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UpdateModelPackageFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::update_model_package::builders::UpdateModelPackageInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::update_model_package::UpdateModelPackageOutput,
        crate::operation::update_model_package::UpdateModelPackageError,
    > for UpdateModelPackageFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::update_model_package::UpdateModelPackageOutput,
            crate::operation::update_model_package::UpdateModelPackageError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UpdateModelPackageFluentBuilder {
    /// Creates a new `UpdateModelPackageFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UpdateModelPackage as a reference.
    pub fn as_input(&self) -> &crate::operation::update_model_package::builders::UpdateModelPackageInputBuilder {
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
        crate::operation::update_model_package::UpdateModelPackageOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_model_package::UpdateModelPackageError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::update_model_package::UpdateModelPackage::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::update_model_package::UpdateModelPackage::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::update_model_package::UpdateModelPackageOutput,
        crate::operation::update_model_package::UpdateModelPackageError,
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
    /// <p>The Amazon Resource Name (ARN) of the model package.</p>
    pub fn model_package_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.model_package_arn(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the model package.</p>
    pub fn set_model_package_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_model_package_arn(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the model package.</p>
    pub fn get_model_package_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_model_package_arn()
    }
    /// <p>The approval status of the model.</p>
    pub fn model_approval_status(mut self, input: crate::types::ModelApprovalStatus) -> Self {
        self.inner = self.inner.model_approval_status(input);
        self
    }
    /// <p>The approval status of the model.</p>
    pub fn set_model_approval_status(mut self, input: ::std::option::Option<crate::types::ModelApprovalStatus>) -> Self {
        self.inner = self.inner.set_model_approval_status(input);
        self
    }
    /// <p>The approval status of the model.</p>
    pub fn get_model_approval_status(&self) -> &::std::option::Option<crate::types::ModelApprovalStatus> {
        self.inner.get_model_approval_status()
    }
    /// <p>A description for the approval status of the model.</p>
    pub fn approval_description(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.approval_description(input.into());
        self
    }
    /// <p>A description for the approval status of the model.</p>
    pub fn set_approval_description(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_approval_description(input);
        self
    }
    /// <p>A description for the approval status of the model.</p>
    pub fn get_approval_description(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_approval_description()
    }
    ///
    /// Adds a key-value pair to `CustomerMetadataProperties`.
    ///
    /// To override the contents of this collection use [`set_customer_metadata_properties`](Self::set_customer_metadata_properties).
    ///
    /// <p>The metadata properties associated with the model package versions.</p>
    pub fn customer_metadata_properties(
        mut self,
        k: impl ::std::convert::Into<::std::string::String>,
        v: impl ::std::convert::Into<::std::string::String>,
    ) -> Self {
        self.inner = self.inner.customer_metadata_properties(k.into(), v.into());
        self
    }
    /// <p>The metadata properties associated with the model package versions.</p>
    pub fn set_customer_metadata_properties(
        mut self,
        input: ::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>>,
    ) -> Self {
        self.inner = self.inner.set_customer_metadata_properties(input);
        self
    }
    /// <p>The metadata properties associated with the model package versions.</p>
    pub fn get_customer_metadata_properties(
        &self,
    ) -> &::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>> {
        self.inner.get_customer_metadata_properties()
    }
    ///
    /// Appends an item to `CustomerMetadataPropertiesToRemove`.
    ///
    /// To override the contents of this collection use [`set_customer_metadata_properties_to_remove`](Self::set_customer_metadata_properties_to_remove).
    ///
    /// <p>The metadata properties associated with the model package versions to remove.</p>
    pub fn customer_metadata_properties_to_remove(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.customer_metadata_properties_to_remove(input.into());
        self
    }
    /// <p>The metadata properties associated with the model package versions to remove.</p>
    pub fn set_customer_metadata_properties_to_remove(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_customer_metadata_properties_to_remove(input);
        self
    }
    /// <p>The metadata properties associated with the model package versions to remove.</p>
    pub fn get_customer_metadata_properties_to_remove(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_customer_metadata_properties_to_remove()
    }
    ///
    /// Appends an item to `AdditionalInferenceSpecificationsToAdd`.
    ///
    /// To override the contents of this collection use [`set_additional_inference_specifications_to_add`](Self::set_additional_inference_specifications_to_add).
    ///
    /// <p>An array of additional Inference Specification objects to be added to the existing array additional Inference Specification. Total number of additional Inference Specifications can not exceed 15. Each additional Inference Specification specifies artifacts based on this model package that can be used on inference endpoints. Generally used with SageMaker Neo to store the compiled artifacts.</p>
    pub fn additional_inference_specifications_to_add(mut self, input: crate::types::AdditionalInferenceSpecificationDefinition) -> Self {
        self.inner = self.inner.additional_inference_specifications_to_add(input);
        self
    }
    /// <p>An array of additional Inference Specification objects to be added to the existing array additional Inference Specification. Total number of additional Inference Specifications can not exceed 15. Each additional Inference Specification specifies artifacts based on this model package that can be used on inference endpoints. Generally used with SageMaker Neo to store the compiled artifacts.</p>
    pub fn set_additional_inference_specifications_to_add(
        mut self,
        input: ::std::option::Option<::std::vec::Vec<crate::types::AdditionalInferenceSpecificationDefinition>>,
    ) -> Self {
        self.inner = self.inner.set_additional_inference_specifications_to_add(input);
        self
    }
    /// <p>An array of additional Inference Specification objects to be added to the existing array additional Inference Specification. Total number of additional Inference Specifications can not exceed 15. Each additional Inference Specification specifies artifacts based on this model package that can be used on inference endpoints. Generally used with SageMaker Neo to store the compiled artifacts.</p>
    pub fn get_additional_inference_specifications_to_add(
        &self,
    ) -> &::std::option::Option<::std::vec::Vec<crate::types::AdditionalInferenceSpecificationDefinition>> {
        self.inner.get_additional_inference_specifications_to_add()
    }
    /// <p>Specifies details about inference jobs that you can run with models based on this model package, including the following information:</p>
    /// <ul>
    /// <li>
    /// <p>The Amazon ECR paths of containers that contain the inference code and model artifacts.</p></li>
    /// <li>
    /// <p>The instance types that the model package supports for transform jobs and real-time endpoints used for inference.</p></li>
    /// <li>
    /// <p>The input and output content formats that the model package supports for inference.</p></li>
    /// </ul>
    pub fn inference_specification(mut self, input: crate::types::InferenceSpecification) -> Self {
        self.inner = self.inner.inference_specification(input);
        self
    }
    /// <p>Specifies details about inference jobs that you can run with models based on this model package, including the following information:</p>
    /// <ul>
    /// <li>
    /// <p>The Amazon ECR paths of containers that contain the inference code and model artifacts.</p></li>
    /// <li>
    /// <p>The instance types that the model package supports for transform jobs and real-time endpoints used for inference.</p></li>
    /// <li>
    /// <p>The input and output content formats that the model package supports for inference.</p></li>
    /// </ul>
    pub fn set_inference_specification(mut self, input: ::std::option::Option<crate::types::InferenceSpecification>) -> Self {
        self.inner = self.inner.set_inference_specification(input);
        self
    }
    /// <p>Specifies details about inference jobs that you can run with models based on this model package, including the following information:</p>
    /// <ul>
    /// <li>
    /// <p>The Amazon ECR paths of containers that contain the inference code and model artifacts.</p></li>
    /// <li>
    /// <p>The instance types that the model package supports for transform jobs and real-time endpoints used for inference.</p></li>
    /// <li>
    /// <p>The input and output content formats that the model package supports for inference.</p></li>
    /// </ul>
    pub fn get_inference_specification(&self) -> &::std::option::Option<crate::types::InferenceSpecification> {
        self.inner.get_inference_specification()
    }
    /// <p>The URI of the source for the model package.</p>
    pub fn source_uri(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.source_uri(input.into());
        self
    }
    /// <p>The URI of the source for the model package.</p>
    pub fn set_source_uri(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_source_uri(input);
        self
    }
    /// <p>The URI of the source for the model package.</p>
    pub fn get_source_uri(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_source_uri()
    }
    /// <p>The model card associated with the model package. Since <code>ModelPackageModelCard</code> is tied to a model package, it is a specific usage of a model card and its schema is simplified compared to the schema of <code>ModelCard</code>. The <code>ModelPackageModelCard</code> schema does not include <code>model_package_details</code>, and <code>model_overview</code> is composed of the <code>model_creator</code> and <code>model_artifact</code> properties. For more information about the model package model card schema, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/model-registry-details.html#model-card-schema">Model package model card schema</a>. For more information about the model card associated with the model package, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/model-registry-details.html">View the Details of a Model Version</a>.</p>
    pub fn model_card(mut self, input: crate::types::ModelPackageModelCard) -> Self {
        self.inner = self.inner.model_card(input);
        self
    }
    /// <p>The model card associated with the model package. Since <code>ModelPackageModelCard</code> is tied to a model package, it is a specific usage of a model card and its schema is simplified compared to the schema of <code>ModelCard</code>. The <code>ModelPackageModelCard</code> schema does not include <code>model_package_details</code>, and <code>model_overview</code> is composed of the <code>model_creator</code> and <code>model_artifact</code> properties. For more information about the model package model card schema, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/model-registry-details.html#model-card-schema">Model package model card schema</a>. For more information about the model card associated with the model package, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/model-registry-details.html">View the Details of a Model Version</a>.</p>
    pub fn set_model_card(mut self, input: ::std::option::Option<crate::types::ModelPackageModelCard>) -> Self {
        self.inner = self.inner.set_model_card(input);
        self
    }
    /// <p>The model card associated with the model package. Since <code>ModelPackageModelCard</code> is tied to a model package, it is a specific usage of a model card and its schema is simplified compared to the schema of <code>ModelCard</code>. The <code>ModelPackageModelCard</code> schema does not include <code>model_package_details</code>, and <code>model_overview</code> is composed of the <code>model_creator</code> and <code>model_artifact</code> properties. For more information about the model package model card schema, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/model-registry-details.html#model-card-schema">Model package model card schema</a>. For more information about the model card associated with the model package, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/model-registry-details.html">View the Details of a Model Version</a>.</p>
    pub fn get_model_card(&self) -> &::std::option::Option<crate::types::ModelPackageModelCard> {
        self.inner.get_model_card()
    }
    /// <p>A structure describing the current state of the model in its life cycle.</p>
    pub fn model_life_cycle(mut self, input: crate::types::ModelLifeCycle) -> Self {
        self.inner = self.inner.model_life_cycle(input);
        self
    }
    /// <p>A structure describing the current state of the model in its life cycle.</p>
    pub fn set_model_life_cycle(mut self, input: ::std::option::Option<crate::types::ModelLifeCycle>) -> Self {
        self.inner = self.inner.set_model_life_cycle(input);
        self
    }
    /// <p>A structure describing the current state of the model in its life cycle.</p>
    pub fn get_model_life_cycle(&self) -> &::std::option::Option<crate::types::ModelLifeCycle> {
        self.inner.get_model_life_cycle()
    }
    /// <p>A unique token that guarantees that the call to this API is idempotent.</p>
    pub fn client_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_token(input.into());
        self
    }
    /// <p>A unique token that guarantees that the call to this API is idempotent.</p>
    pub fn set_client_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_token(input);
        self
    }
    /// <p>A unique token that guarantees that the call to this API is idempotent.</p>
    pub fn get_client_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_token()
    }
}
