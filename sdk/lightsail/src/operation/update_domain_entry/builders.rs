// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::update_domain_entry::_update_domain_entry_output::UpdateDomainEntryOutputBuilder;

pub use crate::operation::update_domain_entry::_update_domain_entry_input::UpdateDomainEntryInputBuilder;

impl crate::operation::update_domain_entry::builders::UpdateDomainEntryInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::update_domain_entry::UpdateDomainEntryOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_domain_entry::UpdateDomainEntryError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.update_domain_entry();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UpdateDomainEntry`.
///
/// <p>Updates a domain recordset after it is created.</p>
/// <p>The <code>update domain entry</code> operation supports tag-based access control via resource tags applied to the resource identified by <code>domain name</code>. For more information, see the <a href="https://docs.aws.amazon.com/lightsail/latest/userguide/amazon-lightsail-controlling-access-using-tags">Amazon Lightsail Developer Guide</a>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UpdateDomainEntryFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::update_domain_entry::builders::UpdateDomainEntryInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::update_domain_entry::UpdateDomainEntryOutput,
        crate::operation::update_domain_entry::UpdateDomainEntryError,
    > for UpdateDomainEntryFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::update_domain_entry::UpdateDomainEntryOutput,
            crate::operation::update_domain_entry::UpdateDomainEntryError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UpdateDomainEntryFluentBuilder {
    /// Creates a new `UpdateDomainEntryFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the UpdateDomainEntry as a reference.
    pub fn as_input(&self) -> &crate::operation::update_domain_entry::builders::UpdateDomainEntryInputBuilder {
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
        crate::operation::update_domain_entry::UpdateDomainEntryOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::update_domain_entry::UpdateDomainEntryError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::update_domain_entry::UpdateDomainEntry::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::update_domain_entry::UpdateDomainEntry::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::update_domain_entry::UpdateDomainEntryOutput,
        crate::operation::update_domain_entry::UpdateDomainEntryError,
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
    /// <p>The name of the domain recordset to update.</p>
    pub fn domain_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.domain_name(input.into());
        self
    }
    /// <p>The name of the domain recordset to update.</p>
    pub fn set_domain_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_domain_name(input);
        self
    }
    /// <p>The name of the domain recordset to update.</p>
    pub fn get_domain_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_domain_name()
    }
    /// <p>An array of key-value pairs containing information about the domain entry.</p>
    pub fn domain_entry(mut self, input: crate::types::DomainEntry) -> Self {
        self.inner = self.inner.domain_entry(input);
        self
    }
    /// <p>An array of key-value pairs containing information about the domain entry.</p>
    pub fn set_domain_entry(mut self, input: ::std::option::Option<crate::types::DomainEntry>) -> Self {
        self.inner = self.inner.set_domain_entry(input);
        self
    }
    /// <p>An array of key-value pairs containing information about the domain entry.</p>
    pub fn get_domain_entry(&self) -> &::std::option::Option<crate::types::DomainEntry> {
        self.inner.get_domain_entry()
    }
}
