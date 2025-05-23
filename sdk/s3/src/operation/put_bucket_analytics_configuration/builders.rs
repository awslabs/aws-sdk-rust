// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::put_bucket_analytics_configuration::_put_bucket_analytics_configuration_output::PutBucketAnalyticsConfigurationOutputBuilder;

pub use crate::operation::put_bucket_analytics_configuration::_put_bucket_analytics_configuration_input::PutBucketAnalyticsConfigurationInputBuilder;

impl crate::operation::put_bucket_analytics_configuration::builders::PutBucketAnalyticsConfigurationInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfigurationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfigurationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.put_bucket_analytics_configuration();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `PutBucketAnalyticsConfiguration`.
///
/// <note>
/// <p>This operation is not supported for directory buckets.</p>
/// </note>
/// <p>Sets an analytics configuration for the bucket (specified by the analytics configuration ID). You can have up to 1,000 analytics configurations per bucket.</p>
/// <p>You can choose to have storage class analysis export analysis reports sent to a comma-separated values (CSV) flat file. See the <code>DataExport</code> request element. Reports are updated daily and are based on the object filters that you configure. When selecting data export, you specify a destination bucket and an optional destination prefix where the file is written. You can export the data to a destination bucket in a different account. However, the destination bucket must be in the same Region as the bucket that you are making the PUT analytics configuration to. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/analytics-storage-class.html">Amazon S3 Analytics – Storage Class Analysis</a>.</p><important>
/// <p>You must create a bucket policy on the destination bucket where the exported file is written to grant permissions to Amazon S3 to write objects to the bucket. For an example policy, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/dev/example-bucket-policies.html#example-bucket-policies-use-case-9">Granting Permissions for Amazon S3 Inventory and Storage Class Analysis</a>.</p>
/// </important>
/// <p>To use this operation, you must have permissions to perform the <code>s3:PutAnalyticsConfiguration</code> action. The bucket owner has this permission by default. The bucket owner can grant this permission to others. For more information about permissions, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/userguide/using-with-s3-actions.html#using-with-s3-actions-related-to-bucket-subresources">Permissions Related to Bucket Subresource Operations</a> and <a href="https://docs.aws.amazon.com/AmazonS3/latest/userguide/s3-access-control.html">Managing Access Permissions to Your Amazon S3 Resources</a>.</p>
/// <p><code>PutBucketAnalyticsConfiguration</code> has the following special errors:</p>
/// <ul>
/// <li>
/// <ul>
/// <li>
/// <p><i>HTTP Error: HTTP 400 Bad Request</i></p></li>
/// <li>
/// <p><i>Code: InvalidArgument</i></p></li>
/// <li>
/// <p><i>Cause: Invalid argument.</i></p></li>
/// </ul></li>
/// <li>
/// <ul>
/// <li>
/// <p><i>HTTP Error: HTTP 400 Bad Request</i></p></li>
/// <li>
/// <p><i>Code: TooManyConfigurations</i></p></li>
/// <li>
/// <p><i>Cause: You are attempting to create a new configuration but have already reached the 1,000-configuration limit.</i></p></li>
/// </ul></li>
/// <li>
/// <ul>
/// <li>
/// <p><i>HTTP Error: HTTP 403 Forbidden</i></p></li>
/// <li>
/// <p><i>Code: AccessDenied</i></p></li>
/// <li>
/// <p><i>Cause: You are not the owner of the specified bucket, or you do not have the s3:PutAnalyticsConfiguration bucket permission to set the configuration on the bucket.</i></p></li>
/// </ul></li>
/// </ul>
/// <p>The following operations are related to <code>PutBucketAnalyticsConfiguration</code>:</p>
/// <ul>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketAnalyticsConfiguration.html">GetBucketAnalyticsConfiguration</a></p></li>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketAnalyticsConfiguration.html">DeleteBucketAnalyticsConfiguration</a></p></li>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBucketAnalyticsConfigurations.html">ListBucketAnalyticsConfigurations</a></p></li>
/// </ul>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct PutBucketAnalyticsConfigurationFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::put_bucket_analytics_configuration::builders::PutBucketAnalyticsConfigurationInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfigurationOutput,
        crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfigurationError,
    > for PutBucketAnalyticsConfigurationFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfigurationOutput,
            crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfigurationError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl PutBucketAnalyticsConfigurationFluentBuilder {
    /// Creates a new `PutBucketAnalyticsConfigurationFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the PutBucketAnalyticsConfiguration as a reference.
    pub fn as_input(&self) -> &crate::operation::put_bucket_analytics_configuration::builders::PutBucketAnalyticsConfigurationInputBuilder {
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
        crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfigurationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfigurationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfiguration::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfiguration::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfigurationOutput,
        crate::operation::put_bucket_analytics_configuration::PutBucketAnalyticsConfigurationError,
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
    /// <p>The name of the bucket to which an analytics configuration is stored.</p>
    pub fn bucket(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.bucket(input.into());
        self
    }
    /// <p>The name of the bucket to which an analytics configuration is stored.</p>
    pub fn set_bucket(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_bucket(input);
        self
    }
    /// <p>The name of the bucket to which an analytics configuration is stored.</p>
    pub fn get_bucket(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_bucket()
    }
    /// <p>The ID that identifies the analytics configuration.</p>
    pub fn id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.id(input.into());
        self
    }
    /// <p>The ID that identifies the analytics configuration.</p>
    pub fn set_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_id(input);
        self
    }
    /// <p>The ID that identifies the analytics configuration.</p>
    pub fn get_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_id()
    }
    /// <p>The configuration and any analyses for the analytics filter.</p>
    pub fn analytics_configuration(mut self, input: crate::types::AnalyticsConfiguration) -> Self {
        self.inner = self.inner.analytics_configuration(input);
        self
    }
    /// <p>The configuration and any analyses for the analytics filter.</p>
    pub fn set_analytics_configuration(mut self, input: ::std::option::Option<crate::types::AnalyticsConfiguration>) -> Self {
        self.inner = self.inner.set_analytics_configuration(input);
        self
    }
    /// <p>The configuration and any analyses for the analytics filter.</p>
    pub fn get_analytics_configuration(&self) -> &::std::option::Option<crate::types::AnalyticsConfiguration> {
        self.inner.get_analytics_configuration()
    }
    /// <p>The account ID of the expected bucket owner. If the account ID that you provide does not match the actual owner of the bucket, the request fails with the HTTP status code <code>403 Forbidden</code> (access denied).</p>
    pub fn expected_bucket_owner(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.expected_bucket_owner(input.into());
        self
    }
    /// <p>The account ID of the expected bucket owner. If the account ID that you provide does not match the actual owner of the bucket, the request fails with the HTTP status code <code>403 Forbidden</code> (access denied).</p>
    pub fn set_expected_bucket_owner(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_expected_bucket_owner(input);
        self
    }
    /// <p>The account ID of the expected bucket owner. If the account ID that you provide does not match the actual owner of the bucket, the request fails with the HTTP status code <code>403 Forbidden</code> (access denied).</p>
    pub fn get_expected_bucket_owner(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_expected_bucket_owner()
    }
}
