// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>Contains the details for an Outbound EDI capability.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct CapabilityOptions {
    /// <p>A structure that contains the outbound EDI options.</p>
    pub outbound_edi: ::std::option::Option<crate::types::OutboundEdiOptions>,
}
impl CapabilityOptions {
    /// <p>A structure that contains the outbound EDI options.</p>
    pub fn outbound_edi(&self) -> ::std::option::Option<&crate::types::OutboundEdiOptions> {
        self.outbound_edi.as_ref()
    }
}
impl CapabilityOptions {
    /// Creates a new builder-style object to manufacture [`CapabilityOptions`](crate::types::CapabilityOptions).
    pub fn builder() -> crate::types::builders::CapabilityOptionsBuilder {
        crate::types::builders::CapabilityOptionsBuilder::default()
    }
}

/// A builder for [`CapabilityOptions`](crate::types::CapabilityOptions).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct CapabilityOptionsBuilder {
    pub(crate) outbound_edi: ::std::option::Option<crate::types::OutboundEdiOptions>,
}
impl CapabilityOptionsBuilder {
    /// <p>A structure that contains the outbound EDI options.</p>
    pub fn outbound_edi(mut self, input: crate::types::OutboundEdiOptions) -> Self {
        self.outbound_edi = ::std::option::Option::Some(input);
        self
    }
    /// <p>A structure that contains the outbound EDI options.</p>
    pub fn set_outbound_edi(mut self, input: ::std::option::Option<crate::types::OutboundEdiOptions>) -> Self {
        self.outbound_edi = input;
        self
    }
    /// <p>A structure that contains the outbound EDI options.</p>
    pub fn get_outbound_edi(&self) -> &::std::option::Option<crate::types::OutboundEdiOptions> {
        &self.outbound_edi
    }
    /// Consumes the builder and constructs a [`CapabilityOptions`](crate::types::CapabilityOptions).
    pub fn build(self) -> crate::types::CapabilityOptions {
        crate::types::CapabilityOptions {
            outbound_edi: self.outbound_edi,
        }
    }
}
