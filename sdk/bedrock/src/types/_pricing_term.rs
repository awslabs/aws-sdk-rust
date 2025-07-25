// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>Describes the usage-based pricing term.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct PricingTerm {
    /// <p>Describes a usage price for each dimension.</p>
    pub rate_card: ::std::vec::Vec<crate::types::DimensionalPriceRate>,
}
impl PricingTerm {
    /// <p>Describes a usage price for each dimension.</p>
    pub fn rate_card(&self) -> &[crate::types::DimensionalPriceRate] {
        use std::ops::Deref;
        self.rate_card.deref()
    }
}
impl PricingTerm {
    /// Creates a new builder-style object to manufacture [`PricingTerm`](crate::types::PricingTerm).
    pub fn builder() -> crate::types::builders::PricingTermBuilder {
        crate::types::builders::PricingTermBuilder::default()
    }
}

/// A builder for [`PricingTerm`](crate::types::PricingTerm).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct PricingTermBuilder {
    pub(crate) rate_card: ::std::option::Option<::std::vec::Vec<crate::types::DimensionalPriceRate>>,
}
impl PricingTermBuilder {
    /// Appends an item to `rate_card`.
    ///
    /// To override the contents of this collection use [`set_rate_card`](Self::set_rate_card).
    ///
    /// <p>Describes a usage price for each dimension.</p>
    pub fn rate_card(mut self, input: crate::types::DimensionalPriceRate) -> Self {
        let mut v = self.rate_card.unwrap_or_default();
        v.push(input);
        self.rate_card = ::std::option::Option::Some(v);
        self
    }
    /// <p>Describes a usage price for each dimension.</p>
    pub fn set_rate_card(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::DimensionalPriceRate>>) -> Self {
        self.rate_card = input;
        self
    }
    /// <p>Describes a usage price for each dimension.</p>
    pub fn get_rate_card(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::DimensionalPriceRate>> {
        &self.rate_card
    }
    /// Consumes the builder and constructs a [`PricingTerm`](crate::types::PricingTerm).
    /// This method will fail if any of the following fields are not set:
    /// - [`rate_card`](crate::types::builders::PricingTermBuilder::rate_card)
    pub fn build(self) -> ::std::result::Result<crate::types::PricingTerm, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::PricingTerm {
            rate_card: self.rate_card.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "rate_card",
                    "rate_card was not specified but it is required when building PricingTerm",
                )
            })?,
        })
    }
}
