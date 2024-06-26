// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_mlflow_tracking_server::_create_mlflow_tracking_server_output::CreateMlflowTrackingServerOutputBuilder;

pub use crate::operation::create_mlflow_tracking_server::_create_mlflow_tracking_server_input::CreateMlflowTrackingServerInputBuilder;

impl crate::operation::create_mlflow_tracking_server::builders::CreateMlflowTrackingServerInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServerOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServerError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_mlflow_tracking_server();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateMlflowTrackingServer`.
///
/// <p>Creates an MLflow Tracking Server using a general purpose Amazon S3 bucket as the artifact store. For more information, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow-create-tracking-server.html">Create an MLflow Tracking Server</a>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateMlflowTrackingServerFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_mlflow_tracking_server::builders::CreateMlflowTrackingServerInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServerOutput,
        crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServerError,
    > for CreateMlflowTrackingServerFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServerOutput,
            crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServerError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateMlflowTrackingServerFluentBuilder {
    /// Creates a new `CreateMlflowTrackingServerFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateMlflowTrackingServer as a reference.
    pub fn as_input(&self) -> &crate::operation::create_mlflow_tracking_server::builders::CreateMlflowTrackingServerInputBuilder {
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
        crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServerOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServerError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServer::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServer::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServerOutput,
        crate::operation::create_mlflow_tracking_server::CreateMlflowTrackingServerError,
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
    /// <p>A unique string identifying the tracking server name. This string is part of the tracking server ARN.</p>
    pub fn tracking_server_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.tracking_server_name(input.into());
        self
    }
    /// <p>A unique string identifying the tracking server name. This string is part of the tracking server ARN.</p>
    pub fn set_tracking_server_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_tracking_server_name(input);
        self
    }
    /// <p>A unique string identifying the tracking server name. This string is part of the tracking server ARN.</p>
    pub fn get_tracking_server_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_tracking_server_name()
    }
    /// <p>The S3 URI for a general purpose bucket to use as the MLflow Tracking Server artifact store.</p>
    pub fn artifact_store_uri(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.artifact_store_uri(input.into());
        self
    }
    /// <p>The S3 URI for a general purpose bucket to use as the MLflow Tracking Server artifact store.</p>
    pub fn set_artifact_store_uri(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_artifact_store_uri(input);
        self
    }
    /// <p>The S3 URI for a general purpose bucket to use as the MLflow Tracking Server artifact store.</p>
    pub fn get_artifact_store_uri(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_artifact_store_uri()
    }
    /// <p>The size of the tracking server you want to create. You can choose between <code>"Small"</code>, <code>"Medium"</code>, and <code>"Large"</code>. The default MLflow Tracking Server configuration size is <code>"Small"</code>. You can choose a size depending on the projected use of the tracking server such as the volume of data logged, number of users, and frequency of use.</p>
    /// <p>We recommend using a small tracking server for teams of up to 25 users, a medium tracking server for teams of up to 50 users, and a large tracking server for teams of up to 100 users.</p>
    pub fn tracking_server_size(mut self, input: crate::types::TrackingServerSize) -> Self {
        self.inner = self.inner.tracking_server_size(input);
        self
    }
    /// <p>The size of the tracking server you want to create. You can choose between <code>"Small"</code>, <code>"Medium"</code>, and <code>"Large"</code>. The default MLflow Tracking Server configuration size is <code>"Small"</code>. You can choose a size depending on the projected use of the tracking server such as the volume of data logged, number of users, and frequency of use.</p>
    /// <p>We recommend using a small tracking server for teams of up to 25 users, a medium tracking server for teams of up to 50 users, and a large tracking server for teams of up to 100 users.</p>
    pub fn set_tracking_server_size(mut self, input: ::std::option::Option<crate::types::TrackingServerSize>) -> Self {
        self.inner = self.inner.set_tracking_server_size(input);
        self
    }
    /// <p>The size of the tracking server you want to create. You can choose between <code>"Small"</code>, <code>"Medium"</code>, and <code>"Large"</code>. The default MLflow Tracking Server configuration size is <code>"Small"</code>. You can choose a size depending on the projected use of the tracking server such as the volume of data logged, number of users, and frequency of use.</p>
    /// <p>We recommend using a small tracking server for teams of up to 25 users, a medium tracking server for teams of up to 50 users, and a large tracking server for teams of up to 100 users.</p>
    pub fn get_tracking_server_size(&self) -> &::std::option::Option<crate::types::TrackingServerSize> {
        self.inner.get_tracking_server_size()
    }
    /// <p>The version of MLflow that the tracking server uses. To see which MLflow versions are available to use, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow.html#mlflow-create-tracking-server-how-it-works">How it works</a>.</p>
    pub fn mlflow_version(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.mlflow_version(input.into());
        self
    }
    /// <p>The version of MLflow that the tracking server uses. To see which MLflow versions are available to use, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow.html#mlflow-create-tracking-server-how-it-works">How it works</a>.</p>
    pub fn set_mlflow_version(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_mlflow_version(input);
        self
    }
    /// <p>The version of MLflow that the tracking server uses. To see which MLflow versions are available to use, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow.html#mlflow-create-tracking-server-how-it-works">How it works</a>.</p>
    pub fn get_mlflow_version(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_mlflow_version()
    }
    /// <p>The Amazon Resource Name (ARN) for an IAM role in your account that the MLflow Tracking Server uses to access the artifact store in Amazon S3. The role should have <code>AmazonS3FullAccess</code> permissions. For more information on IAM permissions for tracking server creation, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow-create-tracking-server-iam.html">Set up IAM permissions for MLflow</a>.</p>
    pub fn role_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.role_arn(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) for an IAM role in your account that the MLflow Tracking Server uses to access the artifact store in Amazon S3. The role should have <code>AmazonS3FullAccess</code> permissions. For more information on IAM permissions for tracking server creation, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow-create-tracking-server-iam.html">Set up IAM permissions for MLflow</a>.</p>
    pub fn set_role_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_role_arn(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) for an IAM role in your account that the MLflow Tracking Server uses to access the artifact store in Amazon S3. The role should have <code>AmazonS3FullAccess</code> permissions. For more information on IAM permissions for tracking server creation, see <a href="https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow-create-tracking-server-iam.html">Set up IAM permissions for MLflow</a>.</p>
    pub fn get_role_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_role_arn()
    }
    /// <p>Whether to enable or disable automatic registration of new MLflow models to the SageMaker Model Registry. To enable automatic model registration, set this value to <code>True</code>. To disable automatic model registration, set this value to <code>False</code>. If not specified, <code>AutomaticModelRegistration</code> defaults to <code>False</code>.</p>
    pub fn automatic_model_registration(mut self, input: bool) -> Self {
        self.inner = self.inner.automatic_model_registration(input);
        self
    }
    /// <p>Whether to enable or disable automatic registration of new MLflow models to the SageMaker Model Registry. To enable automatic model registration, set this value to <code>True</code>. To disable automatic model registration, set this value to <code>False</code>. If not specified, <code>AutomaticModelRegistration</code> defaults to <code>False</code>.</p>
    pub fn set_automatic_model_registration(mut self, input: ::std::option::Option<bool>) -> Self {
        self.inner = self.inner.set_automatic_model_registration(input);
        self
    }
    /// <p>Whether to enable or disable automatic registration of new MLflow models to the SageMaker Model Registry. To enable automatic model registration, set this value to <code>True</code>. To disable automatic model registration, set this value to <code>False</code>. If not specified, <code>AutomaticModelRegistration</code> defaults to <code>False</code>.</p>
    pub fn get_automatic_model_registration(&self) -> &::std::option::Option<bool> {
        self.inner.get_automatic_model_registration()
    }
    /// <p>The day and time of the week in Coordinated Universal Time (UTC) 24-hour standard time that weekly maintenance updates are scheduled. For example: TUE:03:30.</p>
    pub fn weekly_maintenance_window_start(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.weekly_maintenance_window_start(input.into());
        self
    }
    /// <p>The day and time of the week in Coordinated Universal Time (UTC) 24-hour standard time that weekly maintenance updates are scheduled. For example: TUE:03:30.</p>
    pub fn set_weekly_maintenance_window_start(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_weekly_maintenance_window_start(input);
        self
    }
    /// <p>The day and time of the week in Coordinated Universal Time (UTC) 24-hour standard time that weekly maintenance updates are scheduled. For example: TUE:03:30.</p>
    pub fn get_weekly_maintenance_window_start(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_weekly_maintenance_window_start()
    }
    ///
    /// Appends an item to `Tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>Tags consisting of key-value pairs used to manage metadata for the tracking server.</p>
    pub fn tags(mut self, input: crate::types::Tag) -> Self {
        self.inner = self.inner.tags(input);
        self
    }
    /// <p>Tags consisting of key-value pairs used to manage metadata for the tracking server.</p>
    pub fn set_tags(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Tag>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>Tags consisting of key-value pairs used to manage metadata for the tracking server.</p>
    pub fn get_tags(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Tag>> {
        self.inner.get_tags()
    }
}
