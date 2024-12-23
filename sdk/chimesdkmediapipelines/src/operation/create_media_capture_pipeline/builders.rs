// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_media_capture_pipeline::_create_media_capture_pipeline_output::CreateMediaCapturePipelineOutputBuilder;

pub use crate::operation::create_media_capture_pipeline::_create_media_capture_pipeline_input::CreateMediaCapturePipelineInputBuilder;

impl crate::operation::create_media_capture_pipeline::builders::CreateMediaCapturePipelineInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_media_capture_pipeline::CreateMediaCapturePipelineOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_media_capture_pipeline::CreateMediaCapturePipelineError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_media_capture_pipeline();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateMediaCapturePipeline`.
///
/// <p>Creates a media pipeline.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateMediaCapturePipelineFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_media_capture_pipeline::builders::CreateMediaCapturePipelineInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_media_capture_pipeline::CreateMediaCapturePipelineOutput,
        crate::operation::create_media_capture_pipeline::CreateMediaCapturePipelineError,
    > for CreateMediaCapturePipelineFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_media_capture_pipeline::CreateMediaCapturePipelineOutput,
            crate::operation::create_media_capture_pipeline::CreateMediaCapturePipelineError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateMediaCapturePipelineFluentBuilder {
    /// Creates a new `CreateMediaCapturePipelineFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateMediaCapturePipeline as a reference.
    pub fn as_input(&self) -> &crate::operation::create_media_capture_pipeline::builders::CreateMediaCapturePipelineInputBuilder {
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
        crate::operation::create_media_capture_pipeline::CreateMediaCapturePipelineOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_media_capture_pipeline::CreateMediaCapturePipelineError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_media_capture_pipeline::CreateMediaCapturePipeline::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_media_capture_pipeline::CreateMediaCapturePipeline::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_media_capture_pipeline::CreateMediaCapturePipelineOutput,
        crate::operation::create_media_capture_pipeline::CreateMediaCapturePipelineError,
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
    /// <p>Source type from which the media artifacts are captured. A Chime SDK Meeting is the only supported source.</p>
    pub fn source_type(mut self, input: crate::types::MediaPipelineSourceType) -> Self {
        self.inner = self.inner.source_type(input);
        self
    }
    /// <p>Source type from which the media artifacts are captured. A Chime SDK Meeting is the only supported source.</p>
    pub fn set_source_type(mut self, input: ::std::option::Option<crate::types::MediaPipelineSourceType>) -> Self {
        self.inner = self.inner.set_source_type(input);
        self
    }
    /// <p>Source type from which the media artifacts are captured. A Chime SDK Meeting is the only supported source.</p>
    pub fn get_source_type(&self) -> &::std::option::Option<crate::types::MediaPipelineSourceType> {
        self.inner.get_source_type()
    }
    /// <p>ARN of the source from which the media artifacts are captured.</p>
    pub fn source_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.source_arn(input.into());
        self
    }
    /// <p>ARN of the source from which the media artifacts are captured.</p>
    pub fn set_source_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_source_arn(input);
        self
    }
    /// <p>ARN of the source from which the media artifacts are captured.</p>
    pub fn get_source_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_source_arn()
    }
    /// <p>Destination type to which the media artifacts are saved. You must use an S3 bucket.</p>
    pub fn sink_type(mut self, input: crate::types::MediaPipelineSinkType) -> Self {
        self.inner = self.inner.sink_type(input);
        self
    }
    /// <p>Destination type to which the media artifacts are saved. You must use an S3 bucket.</p>
    pub fn set_sink_type(mut self, input: ::std::option::Option<crate::types::MediaPipelineSinkType>) -> Self {
        self.inner = self.inner.set_sink_type(input);
        self
    }
    /// <p>Destination type to which the media artifacts are saved. You must use an S3 bucket.</p>
    pub fn get_sink_type(&self) -> &::std::option::Option<crate::types::MediaPipelineSinkType> {
        self.inner.get_sink_type()
    }
    /// <p>The ARN of the sink type.</p>
    pub fn sink_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.sink_arn(input.into());
        self
    }
    /// <p>The ARN of the sink type.</p>
    pub fn set_sink_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_sink_arn(input);
        self
    }
    /// <p>The ARN of the sink type.</p>
    pub fn get_sink_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_sink_arn()
    }
    /// <p>The unique identifier for the client request. The token makes the API request idempotent. Use a unique token for each media pipeline request.</p>
    pub fn client_request_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_request_token(input.into());
        self
    }
    /// <p>The unique identifier for the client request. The token makes the API request idempotent. Use a unique token for each media pipeline request.</p>
    pub fn set_client_request_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_request_token(input);
        self
    }
    /// <p>The unique identifier for the client request. The token makes the API request idempotent. Use a unique token for each media pipeline request.</p>
    pub fn get_client_request_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_request_token()
    }
    /// <p>The configuration for a specified media pipeline. <code>SourceType</code> must be <code>ChimeSdkMeeting</code>.</p>
    pub fn chime_sdk_meeting_configuration(mut self, input: crate::types::ChimeSdkMeetingConfiguration) -> Self {
        self.inner = self.inner.chime_sdk_meeting_configuration(input);
        self
    }
    /// <p>The configuration for a specified media pipeline. <code>SourceType</code> must be <code>ChimeSdkMeeting</code>.</p>
    pub fn set_chime_sdk_meeting_configuration(mut self, input: ::std::option::Option<crate::types::ChimeSdkMeetingConfiguration>) -> Self {
        self.inner = self.inner.set_chime_sdk_meeting_configuration(input);
        self
    }
    /// <p>The configuration for a specified media pipeline. <code>SourceType</code> must be <code>ChimeSdkMeeting</code>.</p>
    pub fn get_chime_sdk_meeting_configuration(&self) -> &::std::option::Option<crate::types::ChimeSdkMeetingConfiguration> {
        self.inner.get_chime_sdk_meeting_configuration()
    }
    /// <p>An object that contains server side encryption parameters to be used by media capture pipeline. The parameters can also be used by media concatenation pipeline taking media capture pipeline as a media source.</p>
    pub fn sse_aws_key_management_params(mut self, input: crate::types::SseAwsKeyManagementParams) -> Self {
        self.inner = self.inner.sse_aws_key_management_params(input);
        self
    }
    /// <p>An object that contains server side encryption parameters to be used by media capture pipeline. The parameters can also be used by media concatenation pipeline taking media capture pipeline as a media source.</p>
    pub fn set_sse_aws_key_management_params(mut self, input: ::std::option::Option<crate::types::SseAwsKeyManagementParams>) -> Self {
        self.inner = self.inner.set_sse_aws_key_management_params(input);
        self
    }
    /// <p>An object that contains server side encryption parameters to be used by media capture pipeline. The parameters can also be used by media concatenation pipeline taking media capture pipeline as a media source.</p>
    pub fn get_sse_aws_key_management_params(&self) -> &::std::option::Option<crate::types::SseAwsKeyManagementParams> {
        self.inner.get_sse_aws_key_management_params()
    }
    /// <p>The Amazon Resource Name (ARN) of the sink role to be used with <code>AwsKmsKeyId</code> in <code>SseAwsKeyManagementParams</code>. Can only interact with <code>S3Bucket</code> sink type. The role must belong to the caller’s account and be able to act on behalf of the caller during the API call. All minimum policy permissions requirements for the caller to perform sink-related actions are the same for <code>SinkIamRoleArn</code>.</p>
    /// <p>Additionally, the role must have permission to <code>kms:GenerateDataKey</code> using KMS key supplied as <code>AwsKmsKeyId</code> in <code>SseAwsKeyManagementParams</code>. If media concatenation will be required later, the role must also have permission to <code>kms:Decrypt</code> for the same KMS key.</p>
    pub fn sink_iam_role_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.sink_iam_role_arn(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the sink role to be used with <code>AwsKmsKeyId</code> in <code>SseAwsKeyManagementParams</code>. Can only interact with <code>S3Bucket</code> sink type. The role must belong to the caller’s account and be able to act on behalf of the caller during the API call. All minimum policy permissions requirements for the caller to perform sink-related actions are the same for <code>SinkIamRoleArn</code>.</p>
    /// <p>Additionally, the role must have permission to <code>kms:GenerateDataKey</code> using KMS key supplied as <code>AwsKmsKeyId</code> in <code>SseAwsKeyManagementParams</code>. If media concatenation will be required later, the role must also have permission to <code>kms:Decrypt</code> for the same KMS key.</p>
    pub fn set_sink_iam_role_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_sink_iam_role_arn(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the sink role to be used with <code>AwsKmsKeyId</code> in <code>SseAwsKeyManagementParams</code>. Can only interact with <code>S3Bucket</code> sink type. The role must belong to the caller’s account and be able to act on behalf of the caller during the API call. All minimum policy permissions requirements for the caller to perform sink-related actions are the same for <code>SinkIamRoleArn</code>.</p>
    /// <p>Additionally, the role must have permission to <code>kms:GenerateDataKey</code> using KMS key supplied as <code>AwsKmsKeyId</code> in <code>SseAwsKeyManagementParams</code>. If media concatenation will be required later, the role must also have permission to <code>kms:Decrypt</code> for the same KMS key.</p>
    pub fn get_sink_iam_role_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_sink_iam_role_arn()
    }
    ///
    /// Appends an item to `Tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>The tag key-value pairs.</p>
    pub fn tags(mut self, input: crate::types::Tag) -> Self {
        self.inner = self.inner.tags(input);
        self
    }
    /// <p>The tag key-value pairs.</p>
    pub fn set_tags(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Tag>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>The tag key-value pairs.</p>
    pub fn get_tags(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Tag>> {
        self.inner.get_tags()
    }
}
