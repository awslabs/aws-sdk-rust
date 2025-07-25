// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::update_bucket_metadata_inventory_table_configuration::_update_bucket_metadata_inventory_table_configuration_output::UpdateBucketMetadataInventoryTableConfigurationOutputBuilder;

pub use crate::operation::update_bucket_metadata_inventory_table_configuration::_update_bucket_metadata_inventory_table_configuration_input::UpdateBucketMetadataInventoryTableConfigurationInputBuilder;

impl crate::operation::update_bucket_metadata_inventory_table_configuration::builders::UpdateBucketMetadataInventoryTableConfigurationInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfigurationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfigurationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.update_bucket_metadata_inventory_table_configuration();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UpdateBucketMetadataInventoryTableConfiguration`.
///
/// <p>Enables or disables a live inventory table for an S3 Metadata configuration on a general purpose bucket. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/userguide/metadata-tables-overview.html">Accelerating data discovery with S3 Metadata</a> in the <i>Amazon S3 User Guide</i>.</p>
/// <dl>
/// <dt>
/// Permissions
/// </dt>
/// <dd>
/// <p>To use this operation, you must have the following permissions. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/userguide/metadata-tables-permissions.html">Setting up permissions for configuring metadata tables</a> in the <i>Amazon S3 User Guide</i>.</p>
/// <p>If you want to encrypt your inventory table with server-side encryption with Key Management Service (KMS) keys (SSE-KMS), you need additional permissions in your KMS key policy. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/userguide/metadata-tables-permissions.html"> Setting up permissions for configuring metadata tables</a> in the <i>Amazon S3 User Guide</i>.</p>
/// <ul>
/// <li>
/// <p><code>s3:UpdateBucketMetadataInventoryTableConfiguration</code></p></li>
/// <li>
/// <p><code>s3tables:CreateTableBucket</code></p></li>
/// <li>
/// <p><code>s3tables:CreateNamespace</code></p></li>
/// <li>
/// <p><code>s3tables:GetTable</code></p></li>
/// <li>
/// <p><code>s3tables:CreateTable</code></p></li>
/// <li>
/// <p><code>s3tables:PutTablePolicy</code></p></li>
/// <li>
/// <p><code>s3tables:PutTableEncryption</code></p></li>
/// <li>
/// <p><code>kms:DescribeKey</code></p></li>
/// </ul>
/// </dd>
/// </dl>
/// <p>The following operations are related to <code>UpdateBucketMetadataInventoryTableConfiguration</code>:</p>
/// <ul>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucketMetadataConfiguration.html">CreateBucketMetadataConfiguration</a></p></li>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketMetadataConfiguration.html">DeleteBucketMetadataConfiguration</a></p></li>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketMetadataConfiguration.html">GetBucketMetadataConfiguration</a></p></li>
/// <li>
/// <p><a href="https://docs.aws.amazon.com/AmazonS3/latest/API/API_UpdateBucketMetadataJournalTableConfiguration.html">UpdateBucketMetadataJournalTableConfiguration</a></p></li>
/// </ul>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UpdateBucketMetadataInventoryTableConfigurationFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner:
        crate::operation::update_bucket_metadata_inventory_table_configuration::builders::UpdateBucketMetadataInventoryTableConfigurationInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfigurationOutput,
        crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfigurationError,
    > for UpdateBucketMetadataInventoryTableConfigurationFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfigurationOutput,
            crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfigurationError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UpdateBucketMetadataInventoryTableConfigurationFluentBuilder {
    /// Creates a new `UpdateBucketMetadataInventoryTableConfigurationFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UpdateBucketMetadataInventoryTableConfiguration as a reference.
    pub fn as_input(
        &self,
    ) -> &crate::operation::update_bucket_metadata_inventory_table_configuration::builders::UpdateBucketMetadataInventoryTableConfigurationInputBuilder
    {
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
        crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfigurationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfigurationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfiguration::operation_runtime_plugins(
                            self.handle.runtime_plugins.clone(),
                            &self.handle.conf,
                            self.config_override,
                        );
        crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfiguration::orchestrate(
            &runtime_plugins,
            input,
        )
        .await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfigurationOutput,
        crate::operation::update_bucket_metadata_inventory_table_configuration::UpdateBucketMetadataInventoryTableConfigurationError,
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
    /// <p>The general purpose bucket that corresponds to the metadata configuration that you want to enable or disable an inventory table for.</p>
    pub fn bucket(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.bucket(input.into());
        self
    }
    /// <p>The general purpose bucket that corresponds to the metadata configuration that you want to enable or disable an inventory table for.</p>
    pub fn set_bucket(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_bucket(input);
        self
    }
    /// <p>The general purpose bucket that corresponds to the metadata configuration that you want to enable or disable an inventory table for.</p>
    pub fn get_bucket(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_bucket()
    }
    /// <p>The <code>Content-MD5</code> header for the inventory table configuration.</p>
    pub fn content_md5(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.content_md5(input.into());
        self
    }
    /// <p>The <code>Content-MD5</code> header for the inventory table configuration.</p>
    pub fn set_content_md5(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_content_md5(input);
        self
    }
    /// <p>The <code>Content-MD5</code> header for the inventory table configuration.</p>
    pub fn get_content_md5(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_content_md5()
    }
    /// <p>The checksum algorithm to use with your inventory table configuration.</p>
    pub fn checksum_algorithm(mut self, input: crate::types::ChecksumAlgorithm) -> Self {
        self.inner = self.inner.checksum_algorithm(input);
        self
    }
    /// <p>The checksum algorithm to use with your inventory table configuration.</p>
    pub fn set_checksum_algorithm(mut self, input: ::std::option::Option<crate::types::ChecksumAlgorithm>) -> Self {
        self.inner = self.inner.set_checksum_algorithm(input);
        self
    }
    /// <p>The checksum algorithm to use with your inventory table configuration.</p>
    pub fn get_checksum_algorithm(&self) -> &::std::option::Option<crate::types::ChecksumAlgorithm> {
        self.inner.get_checksum_algorithm()
    }
    /// <p>The contents of your inventory table configuration.</p>
    pub fn inventory_table_configuration(mut self, input: crate::types::InventoryTableConfigurationUpdates) -> Self {
        self.inner = self.inner.inventory_table_configuration(input);
        self
    }
    /// <p>The contents of your inventory table configuration.</p>
    pub fn set_inventory_table_configuration(mut self, input: ::std::option::Option<crate::types::InventoryTableConfigurationUpdates>) -> Self {
        self.inner = self.inner.set_inventory_table_configuration(input);
        self
    }
    /// <p>The contents of your inventory table configuration.</p>
    pub fn get_inventory_table_configuration(&self) -> &::std::option::Option<crate::types::InventoryTableConfigurationUpdates> {
        self.inner.get_inventory_table_configuration()
    }
    /// <p>The expected owner of the general purpose bucket that corresponds to the metadata table configuration that you want to enable or disable an inventory table for.</p>
    pub fn expected_bucket_owner(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.expected_bucket_owner(input.into());
        self
    }
    /// <p>The expected owner of the general purpose bucket that corresponds to the metadata table configuration that you want to enable or disable an inventory table for.</p>
    pub fn set_expected_bucket_owner(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_expected_bucket_owner(input);
        self
    }
    /// <p>The expected owner of the general purpose bucket that corresponds to the metadata table configuration that you want to enable or disable an inventory table for.</p>
    pub fn get_expected_bucket_owner(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_expected_bucket_owner()
    }
}
