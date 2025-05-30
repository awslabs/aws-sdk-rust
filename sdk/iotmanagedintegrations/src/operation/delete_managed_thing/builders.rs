// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::delete_managed_thing::_delete_managed_thing_output::DeleteManagedThingOutputBuilder;

pub use crate::operation::delete_managed_thing::_delete_managed_thing_input::DeleteManagedThingInputBuilder;

impl crate::operation::delete_managed_thing::builders::DeleteManagedThingInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::delete_managed_thing::DeleteManagedThingOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::delete_managed_thing::DeleteManagedThingError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.delete_managed_thing();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `DeleteManagedThing`.
///
/// <p>Delete a managed thing. If a controller is deleted, all of the devices connected to it will have their status changed to <code>PENDING</code>. It is not possible to remove a cloud device.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct DeleteManagedThingFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::delete_managed_thing::builders::DeleteManagedThingInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::delete_managed_thing::DeleteManagedThingOutput,
        crate::operation::delete_managed_thing::DeleteManagedThingError,
    > for DeleteManagedThingFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::delete_managed_thing::DeleteManagedThingOutput,
            crate::operation::delete_managed_thing::DeleteManagedThingError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl DeleteManagedThingFluentBuilder {
    /// Creates a new `DeleteManagedThingFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the DeleteManagedThing as a reference.
    pub fn as_input(&self) -> &crate::operation::delete_managed_thing::builders::DeleteManagedThingInputBuilder {
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
        crate::operation::delete_managed_thing::DeleteManagedThingOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::delete_managed_thing::DeleteManagedThingError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::delete_managed_thing::DeleteManagedThing::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::delete_managed_thing::DeleteManagedThing::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::delete_managed_thing::DeleteManagedThingOutput,
        crate::operation::delete_managed_thing::DeleteManagedThingError,
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
    /// <p>The id of the managed thing.</p>
    pub fn identifier(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.identifier(input.into());
        self
    }
    /// <p>The id of the managed thing.</p>
    pub fn set_identifier(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_identifier(input);
        self
    }
    /// <p>The id of the managed thing.</p>
    pub fn get_identifier(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_identifier()
    }
    /// <p>When set to <code>TRUE</code>, a forceful deteletion of the managed thing will occur. When set to <code>FALSE</code>, a non-forceful deletion of the managed thing will occur.</p>
    pub fn force(mut self, input: bool) -> Self {
        self.inner = self.inner.force(input);
        self
    }
    /// <p>When set to <code>TRUE</code>, a forceful deteletion of the managed thing will occur. When set to <code>FALSE</code>, a non-forceful deletion of the managed thing will occur.</p>
    pub fn set_force(mut self, input: ::std::option::Option<bool>) -> Self {
        self.inner = self.inner.set_force(input);
        self
    }
    /// <p>When set to <code>TRUE</code>, a forceful deteletion of the managed thing will occur. When set to <code>FALSE</code>, a non-forceful deletion of the managed thing will occur.</p>
    pub fn get_force(&self) -> &::std::option::Option<bool> {
        self.inner.get_force()
    }
}
