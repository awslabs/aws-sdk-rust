// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_model_import_job::_create_model_import_job_output::CreateModelImportJobOutputBuilder;

pub use crate::operation::create_model_import_job::_create_model_import_job_input::CreateModelImportJobInputBuilder;

impl crate::operation::create_model_import_job::builders::CreateModelImportJobInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_model_import_job::CreateModelImportJobOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_model_import_job::CreateModelImportJobError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_model_import_job();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateModelImportJob`.
///
/// <p>Creates a model import job to import model that you have customized in other environments, such as Amazon SageMaker. For more information, see <a href="https://docs.aws.amazon.com/bedrock/latest/userguide/model-customization-import-model.html">Import a customized model</a></p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateModelImportJobFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_model_import_job::builders::CreateModelImportJobInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_model_import_job::CreateModelImportJobOutput,
        crate::operation::create_model_import_job::CreateModelImportJobError,
    > for CreateModelImportJobFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_model_import_job::CreateModelImportJobOutput,
            crate::operation::create_model_import_job::CreateModelImportJobError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateModelImportJobFluentBuilder {
    /// Creates a new `CreateModelImportJobFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateModelImportJob as a reference.
    pub fn as_input(&self) -> &crate::operation::create_model_import_job::builders::CreateModelImportJobInputBuilder {
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
        crate::operation::create_model_import_job::CreateModelImportJobOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_model_import_job::CreateModelImportJobError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_model_import_job::CreateModelImportJob::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_model_import_job::CreateModelImportJob::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_model_import_job::CreateModelImportJobOutput,
        crate::operation::create_model_import_job::CreateModelImportJobError,
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
    /// <p>The name of the import job.</p>
    pub fn job_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.job_name(input.into());
        self
    }
    /// <p>The name of the import job.</p>
    pub fn set_job_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_job_name(input);
        self
    }
    /// <p>The name of the import job.</p>
    pub fn get_job_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_job_name()
    }
    /// <p>The name of the imported model.</p>
    pub fn imported_model_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.imported_model_name(input.into());
        self
    }
    /// <p>The name of the imported model.</p>
    pub fn set_imported_model_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_imported_model_name(input);
        self
    }
    /// <p>The name of the imported model.</p>
    pub fn get_imported_model_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_imported_model_name()
    }
    /// <p>The Amazon Resource Name (ARN) of the model import job.</p>
    pub fn role_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.role_arn(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the model import job.</p>
    pub fn set_role_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_role_arn(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the model import job.</p>
    pub fn get_role_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_role_arn()
    }
    /// <p>The data source for the imported model.</p>
    pub fn model_data_source(mut self, input: crate::types::ModelDataSource) -> Self {
        self.inner = self.inner.model_data_source(input);
        self
    }
    /// <p>The data source for the imported model.</p>
    pub fn set_model_data_source(mut self, input: ::std::option::Option<crate::types::ModelDataSource>) -> Self {
        self.inner = self.inner.set_model_data_source(input);
        self
    }
    /// <p>The data source for the imported model.</p>
    pub fn get_model_data_source(&self) -> &::std::option::Option<crate::types::ModelDataSource> {
        self.inner.get_model_data_source()
    }
    ///
    /// Appends an item to `jobTags`.
    ///
    /// To override the contents of this collection use [`set_job_tags`](Self::set_job_tags).
    ///
    /// <p>Tags to attach to this import job.</p>
    pub fn job_tags(mut self, input: crate::types::Tag) -> Self {
        self.inner = self.inner.job_tags(input);
        self
    }
    /// <p>Tags to attach to this import job.</p>
    pub fn set_job_tags(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Tag>>) -> Self {
        self.inner = self.inner.set_job_tags(input);
        self
    }
    /// <p>Tags to attach to this import job.</p>
    pub fn get_job_tags(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Tag>> {
        self.inner.get_job_tags()
    }
    ///
    /// Appends an item to `importedModelTags`.
    ///
    /// To override the contents of this collection use [`set_imported_model_tags`](Self::set_imported_model_tags).
    ///
    /// <p>Tags to attach to the imported model.</p>
    pub fn imported_model_tags(mut self, input: crate::types::Tag) -> Self {
        self.inner = self.inner.imported_model_tags(input);
        self
    }
    /// <p>Tags to attach to the imported model.</p>
    pub fn set_imported_model_tags(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Tag>>) -> Self {
        self.inner = self.inner.set_imported_model_tags(input);
        self
    }
    /// <p>Tags to attach to the imported model.</p>
    pub fn get_imported_model_tags(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Tag>> {
        self.inner.get_imported_model_tags()
    }
    /// <p>A unique, case-sensitive identifier to ensure that the API request completes no more than one time. If this token matches a previous request, Amazon Bedrock ignores the request, but does not return an error. For more information, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Run_Instance_Idempotency.html">Ensuring idempotency</a>.</p>
    pub fn client_request_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_request_token(input.into());
        self
    }
    /// <p>A unique, case-sensitive identifier to ensure that the API request completes no more than one time. If this token matches a previous request, Amazon Bedrock ignores the request, but does not return an error. For more information, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Run_Instance_Idempotency.html">Ensuring idempotency</a>.</p>
    pub fn set_client_request_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_request_token(input);
        self
    }
    /// <p>A unique, case-sensitive identifier to ensure that the API request completes no more than one time. If this token matches a previous request, Amazon Bedrock ignores the request, but does not return an error. For more information, see <a href="https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Run_Instance_Idempotency.html">Ensuring idempotency</a>.</p>
    pub fn get_client_request_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_request_token()
    }
    /// <p>VPC configuration parameters for the private Virtual Private Cloud (VPC) that contains the resources you are using for the import job.</p>
    pub fn vpc_config(mut self, input: crate::types::VpcConfig) -> Self {
        self.inner = self.inner.vpc_config(input);
        self
    }
    /// <p>VPC configuration parameters for the private Virtual Private Cloud (VPC) that contains the resources you are using for the import job.</p>
    pub fn set_vpc_config(mut self, input: ::std::option::Option<crate::types::VpcConfig>) -> Self {
        self.inner = self.inner.set_vpc_config(input);
        self
    }
    /// <p>VPC configuration parameters for the private Virtual Private Cloud (VPC) that contains the resources you are using for the import job.</p>
    pub fn get_vpc_config(&self) -> &::std::option::Option<crate::types::VpcConfig> {
        self.inner.get_vpc_config()
    }
    /// <p>The imported model is encrypted at rest using this key.</p>
    pub fn imported_model_kms_key_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.imported_model_kms_key_id(input.into());
        self
    }
    /// <p>The imported model is encrypted at rest using this key.</p>
    pub fn set_imported_model_kms_key_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_imported_model_kms_key_id(input);
        self
    }
    /// <p>The imported model is encrypted at rest using this key.</p>
    pub fn get_imported_model_kms_key_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_imported_model_kms_key_id()
    }
}
