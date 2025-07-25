// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>The journal table configuration for an S3 Metadata configuration.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct JournalTableConfiguration {
    /// <p>The journal table record expiration settings for the journal table.</p>
    pub record_expiration: ::std::option::Option<crate::types::RecordExpiration>,
    /// <p>The encryption configuration for the journal table.</p>
    pub encryption_configuration: ::std::option::Option<crate::types::MetadataTableEncryptionConfiguration>,
}
impl JournalTableConfiguration {
    /// <p>The journal table record expiration settings for the journal table.</p>
    pub fn record_expiration(&self) -> ::std::option::Option<&crate::types::RecordExpiration> {
        self.record_expiration.as_ref()
    }
    /// <p>The encryption configuration for the journal table.</p>
    pub fn encryption_configuration(&self) -> ::std::option::Option<&crate::types::MetadataTableEncryptionConfiguration> {
        self.encryption_configuration.as_ref()
    }
}
impl JournalTableConfiguration {
    /// Creates a new builder-style object to manufacture [`JournalTableConfiguration`](crate::types::JournalTableConfiguration).
    pub fn builder() -> crate::types::builders::JournalTableConfigurationBuilder {
        crate::types::builders::JournalTableConfigurationBuilder::default()
    }
}

/// A builder for [`JournalTableConfiguration`](crate::types::JournalTableConfiguration).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct JournalTableConfigurationBuilder {
    pub(crate) record_expiration: ::std::option::Option<crate::types::RecordExpiration>,
    pub(crate) encryption_configuration: ::std::option::Option<crate::types::MetadataTableEncryptionConfiguration>,
}
impl JournalTableConfigurationBuilder {
    /// <p>The journal table record expiration settings for the journal table.</p>
    /// This field is required.
    pub fn record_expiration(mut self, input: crate::types::RecordExpiration) -> Self {
        self.record_expiration = ::std::option::Option::Some(input);
        self
    }
    /// <p>The journal table record expiration settings for the journal table.</p>
    pub fn set_record_expiration(mut self, input: ::std::option::Option<crate::types::RecordExpiration>) -> Self {
        self.record_expiration = input;
        self
    }
    /// <p>The journal table record expiration settings for the journal table.</p>
    pub fn get_record_expiration(&self) -> &::std::option::Option<crate::types::RecordExpiration> {
        &self.record_expiration
    }
    /// <p>The encryption configuration for the journal table.</p>
    pub fn encryption_configuration(mut self, input: crate::types::MetadataTableEncryptionConfiguration) -> Self {
        self.encryption_configuration = ::std::option::Option::Some(input);
        self
    }
    /// <p>The encryption configuration for the journal table.</p>
    pub fn set_encryption_configuration(mut self, input: ::std::option::Option<crate::types::MetadataTableEncryptionConfiguration>) -> Self {
        self.encryption_configuration = input;
        self
    }
    /// <p>The encryption configuration for the journal table.</p>
    pub fn get_encryption_configuration(&self) -> &::std::option::Option<crate::types::MetadataTableEncryptionConfiguration> {
        &self.encryption_configuration
    }
    /// Consumes the builder and constructs a [`JournalTableConfiguration`](crate::types::JournalTableConfiguration).
    pub fn build(self) -> crate::types::JournalTableConfiguration {
        crate::types::JournalTableConfiguration {
            record_expiration: self.record_expiration,
            encryption_configuration: self.encryption_configuration,
        }
    }
}
