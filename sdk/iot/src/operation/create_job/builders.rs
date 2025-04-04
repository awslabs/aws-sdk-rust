// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_job::_create_job_output::CreateJobOutputBuilder;

pub use crate::operation::create_job::_create_job_input::CreateJobInputBuilder;

impl crate::operation::create_job::builders::CreateJobInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_job::CreateJobOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_job::CreateJobError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_job();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateJob`.
///
/// <p>Creates a job.</p>
/// <p>Requires permission to access the <a href="https://docs.aws.amazon.com/service-authorization/latest/reference/list_awsiot.html#awsiot-actions-as-permissions">CreateJob</a> action.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateJobFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_job::builders::CreateJobInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl crate::client::customize::internal::CustomizableSend<crate::operation::create_job::CreateJobOutput, crate::operation::create_job::CreateJobError>
    for CreateJobFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<crate::operation::create_job::CreateJobOutput, crate::operation::create_job::CreateJobError>,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateJobFluentBuilder {
    /// Creates a new `CreateJobFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateJob as a reference.
    pub fn as_input(&self) -> &crate::operation::create_job::builders::CreateJobInputBuilder {
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
        crate::operation::create_job::CreateJobOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_job::CreateJobError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_job::CreateJob::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_job::CreateJob::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_job::CreateJobOutput,
        crate::operation::create_job::CreateJobError,
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
    /// <p>A job identifier which must be unique for your account. We recommend using a UUID. Alpha-numeric characters, "-" and "_" are valid for use here.</p>
    pub fn job_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.job_id(input.into());
        self
    }
    /// <p>A job identifier which must be unique for your account. We recommend using a UUID. Alpha-numeric characters, "-" and "_" are valid for use here.</p>
    pub fn set_job_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_job_id(input);
        self
    }
    /// <p>A job identifier which must be unique for your account. We recommend using a UUID. Alpha-numeric characters, "-" and "_" are valid for use here.</p>
    pub fn get_job_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_job_id()
    }
    ///
    /// Appends an item to `targets`.
    ///
    /// To override the contents of this collection use [`set_targets`](Self::set_targets).
    ///
    /// <p>A list of things and thing groups to which the job should be sent.</p>
    pub fn targets(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.targets(input.into());
        self
    }
    /// <p>A list of things and thing groups to which the job should be sent.</p>
    pub fn set_targets(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_targets(input);
        self
    }
    /// <p>A list of things and thing groups to which the job should be sent.</p>
    pub fn get_targets(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_targets()
    }
    /// <p>An S3 link, or S3 object URL, to the job document. The link is an Amazon S3 object URL and is required if you don't specify a value for <code>document</code>.</p>
    /// <p>For example, <code>--document-source https://s3.<i>region-code</i>.amazonaws.com/example-firmware/device-firmware.1.0</code></p>
    /// <p>For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/userguide/access-bucket-intro.html">Methods for accessing a bucket</a>.</p>
    pub fn document_source(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.document_source(input.into());
        self
    }
    /// <p>An S3 link, or S3 object URL, to the job document. The link is an Amazon S3 object URL and is required if you don't specify a value for <code>document</code>.</p>
    /// <p>For example, <code>--document-source https://s3.<i>region-code</i>.amazonaws.com/example-firmware/device-firmware.1.0</code></p>
    /// <p>For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/userguide/access-bucket-intro.html">Methods for accessing a bucket</a>.</p>
    pub fn set_document_source(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_document_source(input);
        self
    }
    /// <p>An S3 link, or S3 object URL, to the job document. The link is an Amazon S3 object URL and is required if you don't specify a value for <code>document</code>.</p>
    /// <p>For example, <code>--document-source https://s3.<i>region-code</i>.amazonaws.com/example-firmware/device-firmware.1.0</code></p>
    /// <p>For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/userguide/access-bucket-intro.html">Methods for accessing a bucket</a>.</p>
    pub fn get_document_source(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_document_source()
    }
    /// <p>The job document. Required if you don't specify a value for <code>documentSource</code>.</p>
    pub fn document(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.document(input.into());
        self
    }
    /// <p>The job document. Required if you don't specify a value for <code>documentSource</code>.</p>
    pub fn set_document(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_document(input);
        self
    }
    /// <p>The job document. Required if you don't specify a value for <code>documentSource</code>.</p>
    pub fn get_document(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_document()
    }
    /// <p>A short text description of the job.</p>
    pub fn description(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.description(input.into());
        self
    }
    /// <p>A short text description of the job.</p>
    pub fn set_description(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_description(input);
        self
    }
    /// <p>A short text description of the job.</p>
    pub fn get_description(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_description()
    }
    /// <p>Configuration information for pre-signed S3 URLs.</p>
    pub fn presigned_url_config(mut self, input: crate::types::PresignedUrlConfig) -> Self {
        self.inner = self.inner.presigned_url_config(input);
        self
    }
    /// <p>Configuration information for pre-signed S3 URLs.</p>
    pub fn set_presigned_url_config(mut self, input: ::std::option::Option<crate::types::PresignedUrlConfig>) -> Self {
        self.inner = self.inner.set_presigned_url_config(input);
        self
    }
    /// <p>Configuration information for pre-signed S3 URLs.</p>
    pub fn get_presigned_url_config(&self) -> &::std::option::Option<crate::types::PresignedUrlConfig> {
        self.inner.get_presigned_url_config()
    }
    /// <p>Specifies whether the job will continue to run (CONTINUOUS), or will be complete after all those things specified as targets have completed the job (SNAPSHOT). If continuous, the job may also be run on a thing when a change is detected in a target. For example, a job will run on a thing when the thing is added to a target group, even after the job was completed by all things originally in the group.</p><note>
    /// <p>We recommend that you use continuous jobs instead of snapshot jobs for dynamic thing group targets. By using continuous jobs, devices that join the group receive the job execution even after the job has been created.</p>
    /// </note>
    pub fn target_selection(mut self, input: crate::types::TargetSelection) -> Self {
        self.inner = self.inner.target_selection(input);
        self
    }
    /// <p>Specifies whether the job will continue to run (CONTINUOUS), or will be complete after all those things specified as targets have completed the job (SNAPSHOT). If continuous, the job may also be run on a thing when a change is detected in a target. For example, a job will run on a thing when the thing is added to a target group, even after the job was completed by all things originally in the group.</p><note>
    /// <p>We recommend that you use continuous jobs instead of snapshot jobs for dynamic thing group targets. By using continuous jobs, devices that join the group receive the job execution even after the job has been created.</p>
    /// </note>
    pub fn set_target_selection(mut self, input: ::std::option::Option<crate::types::TargetSelection>) -> Self {
        self.inner = self.inner.set_target_selection(input);
        self
    }
    /// <p>Specifies whether the job will continue to run (CONTINUOUS), or will be complete after all those things specified as targets have completed the job (SNAPSHOT). If continuous, the job may also be run on a thing when a change is detected in a target. For example, a job will run on a thing when the thing is added to a target group, even after the job was completed by all things originally in the group.</p><note>
    /// <p>We recommend that you use continuous jobs instead of snapshot jobs for dynamic thing group targets. By using continuous jobs, devices that join the group receive the job execution even after the job has been created.</p>
    /// </note>
    pub fn get_target_selection(&self) -> &::std::option::Option<crate::types::TargetSelection> {
        self.inner.get_target_selection()
    }
    /// <p>Allows you to create a staged rollout of the job.</p>
    pub fn job_executions_rollout_config(mut self, input: crate::types::JobExecutionsRolloutConfig) -> Self {
        self.inner = self.inner.job_executions_rollout_config(input);
        self
    }
    /// <p>Allows you to create a staged rollout of the job.</p>
    pub fn set_job_executions_rollout_config(mut self, input: ::std::option::Option<crate::types::JobExecutionsRolloutConfig>) -> Self {
        self.inner = self.inner.set_job_executions_rollout_config(input);
        self
    }
    /// <p>Allows you to create a staged rollout of the job.</p>
    pub fn get_job_executions_rollout_config(&self) -> &::std::option::Option<crate::types::JobExecutionsRolloutConfig> {
        self.inner.get_job_executions_rollout_config()
    }
    /// <p>Allows you to create the criteria to abort a job.</p>
    pub fn abort_config(mut self, input: crate::types::AbortConfig) -> Self {
        self.inner = self.inner.abort_config(input);
        self
    }
    /// <p>Allows you to create the criteria to abort a job.</p>
    pub fn set_abort_config(mut self, input: ::std::option::Option<crate::types::AbortConfig>) -> Self {
        self.inner = self.inner.set_abort_config(input);
        self
    }
    /// <p>Allows you to create the criteria to abort a job.</p>
    pub fn get_abort_config(&self) -> &::std::option::Option<crate::types::AbortConfig> {
        self.inner.get_abort_config()
    }
    /// <p>Specifies the amount of time each device has to finish its execution of the job. The timer is started when the job execution status is set to <code>IN_PROGRESS</code>. If the job execution status is not set to another terminal state before the time expires, it will be automatically set to <code>TIMED_OUT</code>.</p>
    pub fn timeout_config(mut self, input: crate::types::TimeoutConfig) -> Self {
        self.inner = self.inner.timeout_config(input);
        self
    }
    /// <p>Specifies the amount of time each device has to finish its execution of the job. The timer is started when the job execution status is set to <code>IN_PROGRESS</code>. If the job execution status is not set to another terminal state before the time expires, it will be automatically set to <code>TIMED_OUT</code>.</p>
    pub fn set_timeout_config(mut self, input: ::std::option::Option<crate::types::TimeoutConfig>) -> Self {
        self.inner = self.inner.set_timeout_config(input);
        self
    }
    /// <p>Specifies the amount of time each device has to finish its execution of the job. The timer is started when the job execution status is set to <code>IN_PROGRESS</code>. If the job execution status is not set to another terminal state before the time expires, it will be automatically set to <code>TIMED_OUT</code>.</p>
    pub fn get_timeout_config(&self) -> &::std::option::Option<crate::types::TimeoutConfig> {
        self.inner.get_timeout_config()
    }
    ///
    /// Appends an item to `tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>Metadata which can be used to manage the job.</p>
    pub fn tags(mut self, input: crate::types::Tag) -> Self {
        self.inner = self.inner.tags(input);
        self
    }
    /// <p>Metadata which can be used to manage the job.</p>
    pub fn set_tags(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Tag>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>Metadata which can be used to manage the job.</p>
    pub fn get_tags(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Tag>> {
        self.inner.get_tags()
    }
    /// <p>The namespace used to indicate that a job is a customer-managed job.</p>
    /// <p>When you specify a value for this parameter, Amazon Web Services IoT Core sends jobs notifications to MQTT topics that contain the value in the following format.</p>
    /// <p><code>$aws/things/<i>THING_NAME</i>/jobs/<i>JOB_ID</i>/notify-namespace-<i>NAMESPACE_ID</i>/</code></p><note>
    /// <p>The <code>namespaceId</code> feature is only supported by IoT Greengrass at this time. For more information, see <a href="https://docs.aws.amazon.com/greengrass/v2/developerguide/setting-up.html">Setting up IoT Greengrass core devices.</a></p>
    /// </note>
    pub fn namespace_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.namespace_id(input.into());
        self
    }
    /// <p>The namespace used to indicate that a job is a customer-managed job.</p>
    /// <p>When you specify a value for this parameter, Amazon Web Services IoT Core sends jobs notifications to MQTT topics that contain the value in the following format.</p>
    /// <p><code>$aws/things/<i>THING_NAME</i>/jobs/<i>JOB_ID</i>/notify-namespace-<i>NAMESPACE_ID</i>/</code></p><note>
    /// <p>The <code>namespaceId</code> feature is only supported by IoT Greengrass at this time. For more information, see <a href="https://docs.aws.amazon.com/greengrass/v2/developerguide/setting-up.html">Setting up IoT Greengrass core devices.</a></p>
    /// </note>
    pub fn set_namespace_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_namespace_id(input);
        self
    }
    /// <p>The namespace used to indicate that a job is a customer-managed job.</p>
    /// <p>When you specify a value for this parameter, Amazon Web Services IoT Core sends jobs notifications to MQTT topics that contain the value in the following format.</p>
    /// <p><code>$aws/things/<i>THING_NAME</i>/jobs/<i>JOB_ID</i>/notify-namespace-<i>NAMESPACE_ID</i>/</code></p><note>
    /// <p>The <code>namespaceId</code> feature is only supported by IoT Greengrass at this time. For more information, see <a href="https://docs.aws.amazon.com/greengrass/v2/developerguide/setting-up.html">Setting up IoT Greengrass core devices.</a></p>
    /// </note>
    pub fn get_namespace_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_namespace_id()
    }
    /// <p>The ARN of the job template used to create the job.</p>
    pub fn job_template_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.job_template_arn(input.into());
        self
    }
    /// <p>The ARN of the job template used to create the job.</p>
    pub fn set_job_template_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_job_template_arn(input);
        self
    }
    /// <p>The ARN of the job template used to create the job.</p>
    pub fn get_job_template_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_job_template_arn()
    }
    /// <p>Allows you to create the criteria to retry a job.</p>
    pub fn job_executions_retry_config(mut self, input: crate::types::JobExecutionsRetryConfig) -> Self {
        self.inner = self.inner.job_executions_retry_config(input);
        self
    }
    /// <p>Allows you to create the criteria to retry a job.</p>
    pub fn set_job_executions_retry_config(mut self, input: ::std::option::Option<crate::types::JobExecutionsRetryConfig>) -> Self {
        self.inner = self.inner.set_job_executions_retry_config(input);
        self
    }
    /// <p>Allows you to create the criteria to retry a job.</p>
    pub fn get_job_executions_retry_config(&self) -> &::std::option::Option<crate::types::JobExecutionsRetryConfig> {
        self.inner.get_job_executions_retry_config()
    }
    ///
    /// Adds a key-value pair to `documentParameters`.
    ///
    /// To override the contents of this collection use [`set_document_parameters`](Self::set_document_parameters).
    ///
    /// <p>Parameters of an Amazon Web Services managed template that you can specify to create the job document.</p><note>
    /// <p><code>documentParameters</code> can only be used when creating jobs from Amazon Web Services managed templates. This parameter can't be used with custom job templates or to create jobs from them.</p>
    /// </note>
    pub fn document_parameters(
        mut self,
        k: impl ::std::convert::Into<::std::string::String>,
        v: impl ::std::convert::Into<::std::string::String>,
    ) -> Self {
        self.inner = self.inner.document_parameters(k.into(), v.into());
        self
    }
    /// <p>Parameters of an Amazon Web Services managed template that you can specify to create the job document.</p><note>
    /// <p><code>documentParameters</code> can only be used when creating jobs from Amazon Web Services managed templates. This parameter can't be used with custom job templates or to create jobs from them.</p>
    /// </note>
    pub fn set_document_parameters(
        mut self,
        input: ::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>>,
    ) -> Self {
        self.inner = self.inner.set_document_parameters(input);
        self
    }
    /// <p>Parameters of an Amazon Web Services managed template that you can specify to create the job document.</p><note>
    /// <p><code>documentParameters</code> can only be used when creating jobs from Amazon Web Services managed templates. This parameter can't be used with custom job templates or to create jobs from them.</p>
    /// </note>
    pub fn get_document_parameters(&self) -> &::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>> {
        self.inner.get_document_parameters()
    }
    /// <p>The configuration that allows you to schedule a job for a future date and time in addition to specifying the end behavior for each job execution.</p>
    pub fn scheduling_config(mut self, input: crate::types::SchedulingConfig) -> Self {
        self.inner = self.inner.scheduling_config(input);
        self
    }
    /// <p>The configuration that allows you to schedule a job for a future date and time in addition to specifying the end behavior for each job execution.</p>
    pub fn set_scheduling_config(mut self, input: ::std::option::Option<crate::types::SchedulingConfig>) -> Self {
        self.inner = self.inner.set_scheduling_config(input);
        self
    }
    /// <p>The configuration that allows you to schedule a job for a future date and time in addition to specifying the end behavior for each job execution.</p>
    pub fn get_scheduling_config(&self) -> &::std::option::Option<crate::types::SchedulingConfig> {
        self.inner.get_scheduling_config()
    }
    ///
    /// Appends an item to `destinationPackageVersions`.
    ///
    /// To override the contents of this collection use [`set_destination_package_versions`](Self::set_destination_package_versions).
    ///
    /// <p>The package version Amazon Resource Names (ARNs) that are installed on the device when the job successfully completes. The package version must be in either the Published or Deprecated state when the job deploys. For more information, see <a href="https://docs.aws.amazon.com/iot/latest/developerguide/preparing-to-use-software-package-catalog.html#package-version-lifecycle">Package version lifecycle</a>.</p>
    /// <p><b>Note:</b>The following Length Constraints relates to a single ARN. Up to 25 package version ARNs are allowed.</p>
    pub fn destination_package_versions(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.destination_package_versions(input.into());
        self
    }
    /// <p>The package version Amazon Resource Names (ARNs) that are installed on the device when the job successfully completes. The package version must be in either the Published or Deprecated state when the job deploys. For more information, see <a href="https://docs.aws.amazon.com/iot/latest/developerguide/preparing-to-use-software-package-catalog.html#package-version-lifecycle">Package version lifecycle</a>.</p>
    /// <p><b>Note:</b>The following Length Constraints relates to a single ARN. Up to 25 package version ARNs are allowed.</p>
    pub fn set_destination_package_versions(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_destination_package_versions(input);
        self
    }
    /// <p>The package version Amazon Resource Names (ARNs) that are installed on the device when the job successfully completes. The package version must be in either the Published or Deprecated state when the job deploys. For more information, see <a href="https://docs.aws.amazon.com/iot/latest/developerguide/preparing-to-use-software-package-catalog.html#package-version-lifecycle">Package version lifecycle</a>.</p>
    /// <p><b>Note:</b>The following Length Constraints relates to a single ARN. Up to 25 package version ARNs are allowed.</p>
    pub fn get_destination_package_versions(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_destination_package_versions()
    }
}
