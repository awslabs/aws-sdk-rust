// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::stop_relational_database::_stop_relational_database_output::StopRelationalDatabaseOutputBuilder;

pub use crate::operation::stop_relational_database::_stop_relational_database_input::StopRelationalDatabaseInputBuilder;

impl crate::operation::stop_relational_database::builders::StopRelationalDatabaseInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::stop_relational_database::StopRelationalDatabaseOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::stop_relational_database::StopRelationalDatabaseError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.stop_relational_database();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `StopRelationalDatabase`.
///
/// <p>Stops a specific database that is currently running in Amazon Lightsail.</p><note>
/// <p>If you don't manually start your database instance after it has been stopped for seven consecutive days, Amazon Lightsail automatically starts it for you. This action helps ensure that your database instance doesn't fall behind on any required maintenance updates.</p>
/// </note>
/// <p>The <code>stop relational database</code> operation supports tag-based access control via resource tags applied to the resource identified by relationalDatabaseName. For more information, see the <a href="https://docs.aws.amazon.com/lightsail/latest/userguide/amazon-lightsail-controlling-access-using-tags">Amazon Lightsail Developer Guide</a>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct StopRelationalDatabaseFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::stop_relational_database::builders::StopRelationalDatabaseInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::stop_relational_database::StopRelationalDatabaseOutput,
        crate::operation::stop_relational_database::StopRelationalDatabaseError,
    > for StopRelationalDatabaseFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::stop_relational_database::StopRelationalDatabaseOutput,
            crate::operation::stop_relational_database::StopRelationalDatabaseError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl StopRelationalDatabaseFluentBuilder {
    /// Creates a new `StopRelationalDatabaseFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the StopRelationalDatabase as a reference.
    pub fn as_input(&self) -> &crate::operation::stop_relational_database::builders::StopRelationalDatabaseInputBuilder {
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
        crate::operation::stop_relational_database::StopRelationalDatabaseOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::stop_relational_database::StopRelationalDatabaseError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::stop_relational_database::StopRelationalDatabase::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::stop_relational_database::StopRelationalDatabase::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::stop_relational_database::StopRelationalDatabaseOutput,
        crate::operation::stop_relational_database::StopRelationalDatabaseError,
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
    /// <p>The name of your database to stop.</p>
    pub fn relational_database_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.relational_database_name(input.into());
        self
    }
    /// <p>The name of your database to stop.</p>
    pub fn set_relational_database_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_relational_database_name(input);
        self
    }
    /// <p>The name of your database to stop.</p>
    pub fn get_relational_database_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_relational_database_name()
    }
    /// <p>The name of your new database snapshot to be created before stopping your database.</p>
    pub fn relational_database_snapshot_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.relational_database_snapshot_name(input.into());
        self
    }
    /// <p>The name of your new database snapshot to be created before stopping your database.</p>
    pub fn set_relational_database_snapshot_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_relational_database_snapshot_name(input);
        self
    }
    /// <p>The name of your new database snapshot to be created before stopping your database.</p>
    pub fn get_relational_database_snapshot_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_relational_database_snapshot_name()
    }
}
