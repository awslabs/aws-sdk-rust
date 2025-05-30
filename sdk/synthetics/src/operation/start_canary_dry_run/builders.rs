// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::start_canary_dry_run::_start_canary_dry_run_output::StartCanaryDryRunOutputBuilder;

pub use crate::operation::start_canary_dry_run::_start_canary_dry_run_input::StartCanaryDryRunInputBuilder;

impl crate::operation::start_canary_dry_run::builders::StartCanaryDryRunInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::start_canary_dry_run::StartCanaryDryRunOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::start_canary_dry_run::StartCanaryDryRunError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.start_canary_dry_run();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `StartCanaryDryRun`.
///
/// <p>Use this operation to start a dry run for a canary that has already been created</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct StartCanaryDryRunFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::start_canary_dry_run::builders::StartCanaryDryRunInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::start_canary_dry_run::StartCanaryDryRunOutput,
        crate::operation::start_canary_dry_run::StartCanaryDryRunError,
    > for StartCanaryDryRunFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::start_canary_dry_run::StartCanaryDryRunOutput,
            crate::operation::start_canary_dry_run::StartCanaryDryRunError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl StartCanaryDryRunFluentBuilder {
    /// Creates a new `StartCanaryDryRunFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the StartCanaryDryRun as a reference.
    pub fn as_input(&self) -> &crate::operation::start_canary_dry_run::builders::StartCanaryDryRunInputBuilder {
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
        crate::operation::start_canary_dry_run::StartCanaryDryRunOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::start_canary_dry_run::StartCanaryDryRunError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::start_canary_dry_run::StartCanaryDryRun::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::start_canary_dry_run::StartCanaryDryRun::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::start_canary_dry_run::StartCanaryDryRunOutput,
        crate::operation::start_canary_dry_run::StartCanaryDryRunError,
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
    /// <p>The name of the canary that you want to dry run. To find canary names, use <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_DescribeCanaries.html">DescribeCanaries</a>.</p>
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.name(input.into());
        self
    }
    /// <p>The name of the canary that you want to dry run. To find canary names, use <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_DescribeCanaries.html">DescribeCanaries</a>.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_name(input);
        self
    }
    /// <p>The name of the canary that you want to dry run. To find canary names, use <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_DescribeCanaries.html">DescribeCanaries</a>.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_name()
    }
    /// <p>Use this structure to input your script code for the canary. This structure contains the Lambda handler with the location where the canary should start running the script. If the script is stored in an Amazon S3 bucket, the bucket name, key, and version are also included. If the script was passed into the canary directly, the script code is contained in the value of <code>Zipfile</code>.</p>
    /// <p>If you are uploading your canary scripts with an Amazon S3 bucket, your zip file should include your script in a certain folder structure.</p>
    /// <ul>
    /// <li>
    /// <p>For Node.js canaries, the folder structure must be <code>nodejs/node_modules/<i>myCanaryFilename.js</i> </code> For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_WritingCanary_Nodejs.html#CloudWatch_Synthetics_Canaries_package">Packaging your Node.js canary files</a></p></li>
    /// <li>
    /// <p>For Python canaries, the folder structure must be <code>python/<i>myCanaryFilename.py</i> </code> or <code>python/<i>myFolder/myCanaryFilename.py</i> </code> For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_WritingCanary_Python.html#CloudWatch_Synthetics_Canaries_WritingCanary_Python_package">Packaging your Python canary files</a></p></li>
    /// </ul>
    pub fn code(mut self, input: crate::types::CanaryCodeInput) -> Self {
        self.inner = self.inner.code(input);
        self
    }
    /// <p>Use this structure to input your script code for the canary. This structure contains the Lambda handler with the location where the canary should start running the script. If the script is stored in an Amazon S3 bucket, the bucket name, key, and version are also included. If the script was passed into the canary directly, the script code is contained in the value of <code>Zipfile</code>.</p>
    /// <p>If you are uploading your canary scripts with an Amazon S3 bucket, your zip file should include your script in a certain folder structure.</p>
    /// <ul>
    /// <li>
    /// <p>For Node.js canaries, the folder structure must be <code>nodejs/node_modules/<i>myCanaryFilename.js</i> </code> For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_WritingCanary_Nodejs.html#CloudWatch_Synthetics_Canaries_package">Packaging your Node.js canary files</a></p></li>
    /// <li>
    /// <p>For Python canaries, the folder structure must be <code>python/<i>myCanaryFilename.py</i> </code> or <code>python/<i>myFolder/myCanaryFilename.py</i> </code> For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_WritingCanary_Python.html#CloudWatch_Synthetics_Canaries_WritingCanary_Python_package">Packaging your Python canary files</a></p></li>
    /// </ul>
    pub fn set_code(mut self, input: ::std::option::Option<crate::types::CanaryCodeInput>) -> Self {
        self.inner = self.inner.set_code(input);
        self
    }
    /// <p>Use this structure to input your script code for the canary. This structure contains the Lambda handler with the location where the canary should start running the script. If the script is stored in an Amazon S3 bucket, the bucket name, key, and version are also included. If the script was passed into the canary directly, the script code is contained in the value of <code>Zipfile</code>.</p>
    /// <p>If you are uploading your canary scripts with an Amazon S3 bucket, your zip file should include your script in a certain folder structure.</p>
    /// <ul>
    /// <li>
    /// <p>For Node.js canaries, the folder structure must be <code>nodejs/node_modules/<i>myCanaryFilename.js</i> </code> For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_WritingCanary_Nodejs.html#CloudWatch_Synthetics_Canaries_package">Packaging your Node.js canary files</a></p></li>
    /// <li>
    /// <p>For Python canaries, the folder structure must be <code>python/<i>myCanaryFilename.py</i> </code> or <code>python/<i>myFolder/myCanaryFilename.py</i> </code> For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_WritingCanary_Python.html#CloudWatch_Synthetics_Canaries_WritingCanary_Python_package">Packaging your Python canary files</a></p></li>
    /// </ul>
    pub fn get_code(&self) -> &::std::option::Option<crate::types::CanaryCodeInput> {
        self.inner.get_code()
    }
    /// <p>Specifies the runtime version to use for the canary. For a list of valid runtime versions and for more information about runtime versions, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_Library.html"> Canary Runtime Versions</a>.</p>
    pub fn runtime_version(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.runtime_version(input.into());
        self
    }
    /// <p>Specifies the runtime version to use for the canary. For a list of valid runtime versions and for more information about runtime versions, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_Library.html"> Canary Runtime Versions</a>.</p>
    pub fn set_runtime_version(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_runtime_version(input);
        self
    }
    /// <p>Specifies the runtime version to use for the canary. For a list of valid runtime versions and for more information about runtime versions, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_Library.html"> Canary Runtime Versions</a>.</p>
    pub fn get_runtime_version(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_runtime_version()
    }
    /// <p>A structure that contains input information for a canary run.</p>
    pub fn run_config(mut self, input: crate::types::CanaryRunConfigInput) -> Self {
        self.inner = self.inner.run_config(input);
        self
    }
    /// <p>A structure that contains input information for a canary run.</p>
    pub fn set_run_config(mut self, input: ::std::option::Option<crate::types::CanaryRunConfigInput>) -> Self {
        self.inner = self.inner.set_run_config(input);
        self
    }
    /// <p>A structure that contains input information for a canary run.</p>
    pub fn get_run_config(&self) -> &::std::option::Option<crate::types::CanaryRunConfigInput> {
        self.inner.get_run_config()
    }
    /// <p>If this canary is to test an endpoint in a VPC, this structure contains information about the subnets and security groups of the VPC endpoint. For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_VPC.html"> Running a Canary in a VPC</a>.</p>
    pub fn vpc_config(mut self, input: crate::types::VpcConfigInput) -> Self {
        self.inner = self.inner.vpc_config(input);
        self
    }
    /// <p>If this canary is to test an endpoint in a VPC, this structure contains information about the subnets and security groups of the VPC endpoint. For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_VPC.html"> Running a Canary in a VPC</a>.</p>
    pub fn set_vpc_config(mut self, input: ::std::option::Option<crate::types::VpcConfigInput>) -> Self {
        self.inner = self.inner.set_vpc_config(input);
        self
    }
    /// <p>If this canary is to test an endpoint in a VPC, this structure contains information about the subnets and security groups of the VPC endpoint. For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_VPC.html"> Running a Canary in a VPC</a>.</p>
    pub fn get_vpc_config(&self) -> &::std::option::Option<crate::types::VpcConfigInput> {
        self.inner.get_vpc_config()
    }
    /// <p>The ARN of the IAM role to be used to run the canary. This role must already exist, and must include <code>lambda.amazonaws.com</code> as a principal in the trust policy. The role must also have the following permissions:</p>
    pub fn execution_role_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.execution_role_arn(input.into());
        self
    }
    /// <p>The ARN of the IAM role to be used to run the canary. This role must already exist, and must include <code>lambda.amazonaws.com</code> as a principal in the trust policy. The role must also have the following permissions:</p>
    pub fn set_execution_role_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_execution_role_arn(input);
        self
    }
    /// <p>The ARN of the IAM role to be used to run the canary. This role must already exist, and must include <code>lambda.amazonaws.com</code> as a principal in the trust policy. The role must also have the following permissions:</p>
    pub fn get_execution_role_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_execution_role_arn()
    }
    /// <p>The number of days to retain data about successful runs of this canary. If you omit this field, the default of 31 days is used. The valid range is 1 to 455 days.</p>
    /// <p>This setting affects the range of information returned by <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_GetCanaryRuns.html">GetCanaryRuns</a>, as well as the range of information displayed in the Synthetics console.</p>
    pub fn success_retention_period_in_days(mut self, input: i32) -> Self {
        self.inner = self.inner.success_retention_period_in_days(input);
        self
    }
    /// <p>The number of days to retain data about successful runs of this canary. If you omit this field, the default of 31 days is used. The valid range is 1 to 455 days.</p>
    /// <p>This setting affects the range of information returned by <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_GetCanaryRuns.html">GetCanaryRuns</a>, as well as the range of information displayed in the Synthetics console.</p>
    pub fn set_success_retention_period_in_days(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_success_retention_period_in_days(input);
        self
    }
    /// <p>The number of days to retain data about successful runs of this canary. If you omit this field, the default of 31 days is used. The valid range is 1 to 455 days.</p>
    /// <p>This setting affects the range of information returned by <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_GetCanaryRuns.html">GetCanaryRuns</a>, as well as the range of information displayed in the Synthetics console.</p>
    pub fn get_success_retention_period_in_days(&self) -> &::std::option::Option<i32> {
        self.inner.get_success_retention_period_in_days()
    }
    /// <p>The number of days to retain data about failed runs of this canary. If you omit this field, the default of 31 days is used. The valid range is 1 to 455 days.</p>
    /// <p>This setting affects the range of information returned by <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_GetCanaryRuns.html">GetCanaryRuns</a>, as well as the range of information displayed in the Synthetics console.</p>
    pub fn failure_retention_period_in_days(mut self, input: i32) -> Self {
        self.inner = self.inner.failure_retention_period_in_days(input);
        self
    }
    /// <p>The number of days to retain data about failed runs of this canary. If you omit this field, the default of 31 days is used. The valid range is 1 to 455 days.</p>
    /// <p>This setting affects the range of information returned by <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_GetCanaryRuns.html">GetCanaryRuns</a>, as well as the range of information displayed in the Synthetics console.</p>
    pub fn set_failure_retention_period_in_days(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_failure_retention_period_in_days(input);
        self
    }
    /// <p>The number of days to retain data about failed runs of this canary. If you omit this field, the default of 31 days is used. The valid range is 1 to 455 days.</p>
    /// <p>This setting affects the range of information returned by <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_GetCanaryRuns.html">GetCanaryRuns</a>, as well as the range of information displayed in the Synthetics console.</p>
    pub fn get_failure_retention_period_in_days(&self) -> &::std::option::Option<i32> {
        self.inner.get_failure_retention_period_in_days()
    }
    /// <p>An object that specifies what screenshots to use as a baseline for visual monitoring by this canary. It can optionally also specify parts of the screenshots to ignore during the visual monitoring comparison.</p>
    /// <p>Visual monitoring is supported only on canaries running the <b>syn-puppeteer-node-3.2</b> runtime or later. For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Library_SyntheticsLogger_VisualTesting.html"> Visual monitoring</a> and <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_Blueprints_VisualTesting.html"> Visual monitoring blueprint</a></p>
    pub fn visual_reference(mut self, input: crate::types::VisualReferenceInput) -> Self {
        self.inner = self.inner.visual_reference(input);
        self
    }
    /// <p>An object that specifies what screenshots to use as a baseline for visual monitoring by this canary. It can optionally also specify parts of the screenshots to ignore during the visual monitoring comparison.</p>
    /// <p>Visual monitoring is supported only on canaries running the <b>syn-puppeteer-node-3.2</b> runtime or later. For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Library_SyntheticsLogger_VisualTesting.html"> Visual monitoring</a> and <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_Blueprints_VisualTesting.html"> Visual monitoring blueprint</a></p>
    pub fn set_visual_reference(mut self, input: ::std::option::Option<crate::types::VisualReferenceInput>) -> Self {
        self.inner = self.inner.set_visual_reference(input);
        self
    }
    /// <p>An object that specifies what screenshots to use as a baseline for visual monitoring by this canary. It can optionally also specify parts of the screenshots to ignore during the visual monitoring comparison.</p>
    /// <p>Visual monitoring is supported only on canaries running the <b>syn-puppeteer-node-3.2</b> runtime or later. For more information, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Library_SyntheticsLogger_VisualTesting.html"> Visual monitoring</a> and <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Synthetics_Canaries_Blueprints_VisualTesting.html"> Visual monitoring blueprint</a></p>
    pub fn get_visual_reference(&self) -> &::std::option::Option<crate::types::VisualReferenceInput> {
        self.inner.get_visual_reference()
    }
    /// <p>The location in Amazon S3 where Synthetics stores artifacts from the test runs of this canary. Artifacts include the log file, screenshots, and HAR files. The name of the Amazon S3 bucket can't include a period (.).</p>
    pub fn artifact_s3_location(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.artifact_s3_location(input.into());
        self
    }
    /// <p>The location in Amazon S3 where Synthetics stores artifacts from the test runs of this canary. Artifacts include the log file, screenshots, and HAR files. The name of the Amazon S3 bucket can't include a period (.).</p>
    pub fn set_artifact_s3_location(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_artifact_s3_location(input);
        self
    }
    /// <p>The location in Amazon S3 where Synthetics stores artifacts from the test runs of this canary. Artifacts include the log file, screenshots, and HAR files. The name of the Amazon S3 bucket can't include a period (.).</p>
    pub fn get_artifact_s3_location(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_artifact_s3_location()
    }
    /// <p>A structure that contains the configuration for canary artifacts, including the encryption-at-rest settings for artifacts that the canary uploads to Amazon S3.</p>
    pub fn artifact_config(mut self, input: crate::types::ArtifactConfigInput) -> Self {
        self.inner = self.inner.artifact_config(input);
        self
    }
    /// <p>A structure that contains the configuration for canary artifacts, including the encryption-at-rest settings for artifacts that the canary uploads to Amazon S3.</p>
    pub fn set_artifact_config(mut self, input: ::std::option::Option<crate::types::ArtifactConfigInput>) -> Self {
        self.inner = self.inner.set_artifact_config(input);
        self
    }
    /// <p>A structure that contains the configuration for canary artifacts, including the encryption-at-rest settings for artifacts that the canary uploads to Amazon S3.</p>
    pub fn get_artifact_config(&self) -> &::std::option::Option<crate::types::ArtifactConfigInput> {
        self.inner.get_artifact_config()
    }
    /// <p>Specifies whether to also delete the Lambda functions and layers used by this canary when the canary is deleted. If you omit this parameter, the default of <code>AUTOMATIC</code> is used, which means that the Lambda functions and layers will be deleted when the canary is deleted.</p>
    /// <p>If the value of this parameter is <code>OFF</code>, then the value of the <code>DeleteLambda</code> parameter of the <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_DeleteCanary.html">DeleteCanary</a> operation determines whether the Lambda functions and layers will be deleted.</p>
    pub fn provisioned_resource_cleanup(mut self, input: crate::types::ProvisionedResourceCleanupSetting) -> Self {
        self.inner = self.inner.provisioned_resource_cleanup(input);
        self
    }
    /// <p>Specifies whether to also delete the Lambda functions and layers used by this canary when the canary is deleted. If you omit this parameter, the default of <code>AUTOMATIC</code> is used, which means that the Lambda functions and layers will be deleted when the canary is deleted.</p>
    /// <p>If the value of this parameter is <code>OFF</code>, then the value of the <code>DeleteLambda</code> parameter of the <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_DeleteCanary.html">DeleteCanary</a> operation determines whether the Lambda functions and layers will be deleted.</p>
    pub fn set_provisioned_resource_cleanup(mut self, input: ::std::option::Option<crate::types::ProvisionedResourceCleanupSetting>) -> Self {
        self.inner = self.inner.set_provisioned_resource_cleanup(input);
        self
    }
    /// <p>Specifies whether to also delete the Lambda functions and layers used by this canary when the canary is deleted. If you omit this parameter, the default of <code>AUTOMATIC</code> is used, which means that the Lambda functions and layers will be deleted when the canary is deleted.</p>
    /// <p>If the value of this parameter is <code>OFF</code>, then the value of the <code>DeleteLambda</code> parameter of the <a href="https://docs.aws.amazon.com/AmazonSynthetics/latest/APIReference/API_DeleteCanary.html">DeleteCanary</a> operation determines whether the Lambda functions and layers will be deleted.</p>
    pub fn get_provisioned_resource_cleanup(&self) -> &::std::option::Option<crate::types::ProvisionedResourceCleanupSetting> {
        self.inner.get_provisioned_resource_cleanup()
    }
}
