// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::request_upload_credentials::_request_upload_credentials_output::RequestUploadCredentialsOutputBuilder;

pub use crate::operation::request_upload_credentials::_request_upload_credentials_input::RequestUploadCredentialsInputBuilder;

impl crate::operation::request_upload_credentials::builders::RequestUploadCredentialsInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::request_upload_credentials::RequestUploadCredentialsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::request_upload_credentials::RequestUploadCredentialsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.request_upload_credentials();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `RequestUploadCredentials`.
///
/// <p>Retrieves a fresh set of credentials for use when uploading a new set of game build files to Amazon GameLift Servers's Amazon S3. This is done as part of the build creation process; see <a href="https://docs.aws.amazon.com/gamelift/latest/apireference/API_CreateBuild.html">CreateBuild</a>.</p>
/// <p>To request new credentials, specify the build ID as returned with an initial <code>CreateBuild</code> request. If successful, a new set of credentials are returned, along with the S3 storage location associated with the build ID.</p>
/// <p><b>Learn more</b></p>
/// <p><a href="https://docs.aws.amazon.com/gamelift/latest/developerguide/gamelift-build-cli-uploading.html#gamelift-build-cli-uploading-create-build"> Create a Build with Files in S3</a></p>
/// <p><a href="https://docs.aws.amazon.com/gamelift/latest/developerguide/reference-awssdk.html#reference-awssdk-resources-fleets">All APIs by task</a></p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct RequestUploadCredentialsFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::request_upload_credentials::builders::RequestUploadCredentialsInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::request_upload_credentials::RequestUploadCredentialsOutput,
        crate::operation::request_upload_credentials::RequestUploadCredentialsError,
    > for RequestUploadCredentialsFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::request_upload_credentials::RequestUploadCredentialsOutput,
            crate::operation::request_upload_credentials::RequestUploadCredentialsError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl RequestUploadCredentialsFluentBuilder {
    /// Creates a new `RequestUploadCredentialsFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the RequestUploadCredentials as a reference.
    pub fn as_input(&self) -> &crate::operation::request_upload_credentials::builders::RequestUploadCredentialsInputBuilder {
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
        crate::operation::request_upload_credentials::RequestUploadCredentialsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::request_upload_credentials::RequestUploadCredentialsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::request_upload_credentials::RequestUploadCredentials::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::request_upload_credentials::RequestUploadCredentials::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::request_upload_credentials::RequestUploadCredentialsOutput,
        crate::operation::request_upload_credentials::RequestUploadCredentialsError,
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
    /// <p>A unique identifier for the build to get credentials for. You can use either the build ID or ARN value.</p>
    pub fn build_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.build_id(input.into());
        self
    }
    /// <p>A unique identifier for the build to get credentials for. You can use either the build ID or ARN value.</p>
    pub fn set_build_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_build_id(input);
        self
    }
    /// <p>A unique identifier for the build to get credentials for. You can use either the build ID or ARN value.</p>
    pub fn get_build_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_build_id()
    }
}
