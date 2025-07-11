// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::describe_create_account_status::_describe_create_account_status_output::DescribeCreateAccountStatusOutputBuilder;

pub use crate::operation::describe_create_account_status::_describe_create_account_status_input::DescribeCreateAccountStatusInputBuilder;

impl crate::operation::describe_create_account_status::builders::DescribeCreateAccountStatusInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::describe_create_account_status::DescribeCreateAccountStatusOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_create_account_status::DescribeCreateAccountStatusError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.describe_create_account_status();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `DescribeCreateAccountStatus`.
///
/// <p>Retrieves the current status of an asynchronous request to create an account.</p>
/// <p>This operation can be called only from the organization's management account or by a member account that is a delegated administrator.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct DescribeCreateAccountStatusFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::describe_create_account_status::builders::DescribeCreateAccountStatusInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::describe_create_account_status::DescribeCreateAccountStatusOutput,
        crate::operation::describe_create_account_status::DescribeCreateAccountStatusError,
    > for DescribeCreateAccountStatusFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::describe_create_account_status::DescribeCreateAccountStatusOutput,
            crate::operation::describe_create_account_status::DescribeCreateAccountStatusError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl DescribeCreateAccountStatusFluentBuilder {
    /// Creates a new `DescribeCreateAccountStatusFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the DescribeCreateAccountStatus as a reference.
    pub fn as_input(&self) -> &crate::operation::describe_create_account_status::builders::DescribeCreateAccountStatusInputBuilder {
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
        crate::operation::describe_create_account_status::DescribeCreateAccountStatusOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_create_account_status::DescribeCreateAccountStatusError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::describe_create_account_status::DescribeCreateAccountStatus::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::describe_create_account_status::DescribeCreateAccountStatus::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::describe_create_account_status::DescribeCreateAccountStatusOutput,
        crate::operation::describe_create_account_status::DescribeCreateAccountStatusError,
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
    /// <p>Specifies the <code>Id</code> value that uniquely identifies the <code>CreateAccount</code> request. You can get the value from the <code>CreateAccountStatus.Id</code> response in an earlier <code>CreateAccount</code> request, or from the <code>ListCreateAccountStatus</code> operation.</p>
    /// <p>The <a href="http://wikipedia.org/wiki/regex">regex pattern</a> for a create account request ID string requires "car-" followed by from 8 to 32 lowercase letters or digits.</p>
    pub fn create_account_request_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.create_account_request_id(input.into());
        self
    }
    /// <p>Specifies the <code>Id</code> value that uniquely identifies the <code>CreateAccount</code> request. You can get the value from the <code>CreateAccountStatus.Id</code> response in an earlier <code>CreateAccount</code> request, or from the <code>ListCreateAccountStatus</code> operation.</p>
    /// <p>The <a href="http://wikipedia.org/wiki/regex">regex pattern</a> for a create account request ID string requires "car-" followed by from 8 to 32 lowercase letters or digits.</p>
    pub fn set_create_account_request_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_create_account_request_id(input);
        self
    }
    /// <p>Specifies the <code>Id</code> value that uniquely identifies the <code>CreateAccount</code> request. You can get the value from the <code>CreateAccountStatus.Id</code> response in an earlier <code>CreateAccount</code> request, or from the <code>ListCreateAccountStatus</code> operation.</p>
    /// <p>The <a href="http://wikipedia.org/wiki/regex">regex pattern</a> for a create account request ID string requires "car-" followed by from 8 to 32 lowercase letters or digits.</p>
    pub fn get_create_account_request_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_create_account_request_id()
    }
}
