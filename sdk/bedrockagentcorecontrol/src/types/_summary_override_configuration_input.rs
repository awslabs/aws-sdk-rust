// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>Input for summary override configuration in a memory strategy.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct SummaryOverrideConfigurationInput {
    /// <p>The consolidation configuration for a summary override.</p>
    pub consolidation: ::std::option::Option<crate::types::SummaryOverrideConsolidationConfigurationInput>,
}
impl SummaryOverrideConfigurationInput {
    /// <p>The consolidation configuration for a summary override.</p>
    pub fn consolidation(&self) -> ::std::option::Option<&crate::types::SummaryOverrideConsolidationConfigurationInput> {
        self.consolidation.as_ref()
    }
}
impl SummaryOverrideConfigurationInput {
    /// Creates a new builder-style object to manufacture [`SummaryOverrideConfigurationInput`](crate::types::SummaryOverrideConfigurationInput).
    pub fn builder() -> crate::types::builders::SummaryOverrideConfigurationInputBuilder {
        crate::types::builders::SummaryOverrideConfigurationInputBuilder::default()
    }
}

/// A builder for [`SummaryOverrideConfigurationInput`](crate::types::SummaryOverrideConfigurationInput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct SummaryOverrideConfigurationInputBuilder {
    pub(crate) consolidation: ::std::option::Option<crate::types::SummaryOverrideConsolidationConfigurationInput>,
}
impl SummaryOverrideConfigurationInputBuilder {
    /// <p>The consolidation configuration for a summary override.</p>
    pub fn consolidation(mut self, input: crate::types::SummaryOverrideConsolidationConfigurationInput) -> Self {
        self.consolidation = ::std::option::Option::Some(input);
        self
    }
    /// <p>The consolidation configuration for a summary override.</p>
    pub fn set_consolidation(mut self, input: ::std::option::Option<crate::types::SummaryOverrideConsolidationConfigurationInput>) -> Self {
        self.consolidation = input;
        self
    }
    /// <p>The consolidation configuration for a summary override.</p>
    pub fn get_consolidation(&self) -> &::std::option::Option<crate::types::SummaryOverrideConsolidationConfigurationInput> {
        &self.consolidation
    }
    /// Consumes the builder and constructs a [`SummaryOverrideConfigurationInput`](crate::types::SummaryOverrideConfigurationInput).
    pub fn build(self) -> crate::types::SummaryOverrideConfigurationInput {
        crate::types::SummaryOverrideConfigurationInput {
            consolidation: self.consolidation,
        }
    }
}
