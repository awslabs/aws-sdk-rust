// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::delete_transformer::_delete_transformer_output::DeleteTransformerOutputBuilder;

pub use crate::operation::delete_transformer::_delete_transformer_input::DeleteTransformerInputBuilder;

impl crate::operation::delete_transformer::builders::DeleteTransformerInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::delete_transformer::DeleteTransformerOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::delete_transformer::DeleteTransformerError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.delete_transformer();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `DeleteTransformer`.
///
/// <p>Deletes the log transformer for the specified log group. As soon as you do this, the transformation of incoming log events according to that transformer stops. If this account has an account-level transformer that applies to this log group, the log group begins using that account-level transformer when this log-group level transformer is deleted.</p>
/// <p>After you delete a transformer, be sure to edit any metric filters or subscription filters that relied on the transformed versions of the log events.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct DeleteTransformerFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::delete_transformer::builders::DeleteTransformerInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::delete_transformer::DeleteTransformerOutput,
        crate::operation::delete_transformer::DeleteTransformerError,
    > for DeleteTransformerFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::delete_transformer::DeleteTransformerOutput,
            crate::operation::delete_transformer::DeleteTransformerError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl DeleteTransformerFluentBuilder {
    /// Creates a new `DeleteTransformerFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the DeleteTransformer as a reference.
    pub fn as_input(&self) -> &crate::operation::delete_transformer::builders::DeleteTransformerInputBuilder {
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
        crate::operation::delete_transformer::DeleteTransformerOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::delete_transformer::DeleteTransformerError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::delete_transformer::DeleteTransformer::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::delete_transformer::DeleteTransformer::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::delete_transformer::DeleteTransformerOutput,
        crate::operation::delete_transformer::DeleteTransformerError,
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
    /// <p>Specify either the name or ARN of the log group to delete the transformer for. If the log group is in a source account and you are using a monitoring account, you must use the log group ARN.</p>
    pub fn log_group_identifier(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.log_group_identifier(input.into());
        self
    }
    /// <p>Specify either the name or ARN of the log group to delete the transformer for. If the log group is in a source account and you are using a monitoring account, you must use the log group ARN.</p>
    pub fn set_log_group_identifier(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_log_group_identifier(input);
        self
    }
    /// <p>Specify either the name or ARN of the log group to delete the transformer for. If the log group is in a source account and you are using a monitoring account, you must use the log group ARN.</p>
    pub fn get_log_group_identifier(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_log_group_identifier()
    }
}
