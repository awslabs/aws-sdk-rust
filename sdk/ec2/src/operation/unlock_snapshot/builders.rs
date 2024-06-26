// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::unlock_snapshot::_unlock_snapshot_output::UnlockSnapshotOutputBuilder;

pub use crate::operation::unlock_snapshot::_unlock_snapshot_input::UnlockSnapshotInputBuilder;

impl crate::operation::unlock_snapshot::builders::UnlockSnapshotInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::unlock_snapshot::UnlockSnapshotOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::unlock_snapshot::UnlockSnapshotError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.unlock_snapshot();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UnlockSnapshot`.
///
/// <p>Unlocks a snapshot that is locked in governance mode or that is locked in compliance mode but still in the cooling-off period. You can't unlock a snapshot that is locked in compliance mode after the cooling-off period has expired.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UnlockSnapshotFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::unlock_snapshot::builders::UnlockSnapshotInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::unlock_snapshot::UnlockSnapshotOutput,
        crate::operation::unlock_snapshot::UnlockSnapshotError,
    > for UnlockSnapshotFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::unlock_snapshot::UnlockSnapshotOutput,
            crate::operation::unlock_snapshot::UnlockSnapshotError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UnlockSnapshotFluentBuilder {
    /// Creates a new `UnlockSnapshotFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UnlockSnapshot as a reference.
    pub fn as_input(&self) -> &crate::operation::unlock_snapshot::builders::UnlockSnapshotInputBuilder {
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
        crate::operation::unlock_snapshot::UnlockSnapshotOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::unlock_snapshot::UnlockSnapshotError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::unlock_snapshot::UnlockSnapshot::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::unlock_snapshot::UnlockSnapshot::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::unlock_snapshot::UnlockSnapshotOutput,
        crate::operation::unlock_snapshot::UnlockSnapshotError,
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
    /// <p>The ID of the snapshot to unlock.</p>
    pub fn snapshot_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.snapshot_id(input.into());
        self
    }
    /// <p>The ID of the snapshot to unlock.</p>
    pub fn set_snapshot_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_snapshot_id(input);
        self
    }
    /// <p>The ID of the snapshot to unlock.</p>
    pub fn get_snapshot_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_snapshot_id()
    }
    /// <p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p>
    pub fn dry_run(mut self, input: bool) -> Self {
        self.inner = self.inner.dry_run(input);
        self
    }
    /// <p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p>
    pub fn set_dry_run(mut self, input: ::std::option::Option<bool>) -> Self {
        self.inner = self.inner.set_dry_run(input);
        self
    }
    /// <p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p>
    pub fn get_dry_run(&self) -> &::std::option::Option<bool> {
        self.inner.get_dry_run()
    }
}
