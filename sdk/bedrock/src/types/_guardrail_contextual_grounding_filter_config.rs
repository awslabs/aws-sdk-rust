// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>The filter configuration details for the guardrails contextual grounding filter.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct GuardrailContextualGroundingFilterConfig {
    /// <p>The filter details for the guardrails contextual grounding filter.</p>
    pub r#type: crate::types::GuardrailContextualGroundingFilterType,
    /// <p>The threshold details for the guardrails contextual grounding filter.</p>
    pub threshold: f64,
}
impl GuardrailContextualGroundingFilterConfig {
    /// <p>The filter details for the guardrails contextual grounding filter.</p>
    pub fn r#type(&self) -> &crate::types::GuardrailContextualGroundingFilterType {
        &self.r#type
    }
    /// <p>The threshold details for the guardrails contextual grounding filter.</p>
    pub fn threshold(&self) -> f64 {
        self.threshold
    }
}
impl GuardrailContextualGroundingFilterConfig {
    /// Creates a new builder-style object to manufacture [`GuardrailContextualGroundingFilterConfig`](crate::types::GuardrailContextualGroundingFilterConfig).
    pub fn builder() -> crate::types::builders::GuardrailContextualGroundingFilterConfigBuilder {
        crate::types::builders::GuardrailContextualGroundingFilterConfigBuilder::default()
    }
}

/// A builder for [`GuardrailContextualGroundingFilterConfig`](crate::types::GuardrailContextualGroundingFilterConfig).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct GuardrailContextualGroundingFilterConfigBuilder {
    pub(crate) r#type: ::std::option::Option<crate::types::GuardrailContextualGroundingFilterType>,
    pub(crate) threshold: ::std::option::Option<f64>,
}
impl GuardrailContextualGroundingFilterConfigBuilder {
    /// <p>The filter details for the guardrails contextual grounding filter.</p>
    /// This field is required.
    pub fn r#type(mut self, input: crate::types::GuardrailContextualGroundingFilterType) -> Self {
        self.r#type = ::std::option::Option::Some(input);
        self
    }
    /// <p>The filter details for the guardrails contextual grounding filter.</p>
    pub fn set_type(mut self, input: ::std::option::Option<crate::types::GuardrailContextualGroundingFilterType>) -> Self {
        self.r#type = input;
        self
    }
    /// <p>The filter details for the guardrails contextual grounding filter.</p>
    pub fn get_type(&self) -> &::std::option::Option<crate::types::GuardrailContextualGroundingFilterType> {
        &self.r#type
    }
    /// <p>The threshold details for the guardrails contextual grounding filter.</p>
    /// This field is required.
    pub fn threshold(mut self, input: f64) -> Self {
        self.threshold = ::std::option::Option::Some(input);
        self
    }
    /// <p>The threshold details for the guardrails contextual grounding filter.</p>
    pub fn set_threshold(mut self, input: ::std::option::Option<f64>) -> Self {
        self.threshold = input;
        self
    }
    /// <p>The threshold details for the guardrails contextual grounding filter.</p>
    pub fn get_threshold(&self) -> &::std::option::Option<f64> {
        &self.threshold
    }
    /// Consumes the builder and constructs a [`GuardrailContextualGroundingFilterConfig`](crate::types::GuardrailContextualGroundingFilterConfig).
    /// This method will fail if any of the following fields are not set:
    /// - [`r#type`](crate::types::builders::GuardrailContextualGroundingFilterConfigBuilder::type)
    /// - [`threshold`](crate::types::builders::GuardrailContextualGroundingFilterConfigBuilder::threshold)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::GuardrailContextualGroundingFilterConfig, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::GuardrailContextualGroundingFilterConfig {
            r#type: self.r#type.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "r#type",
                    "r#type was not specified but it is required when building GuardrailContextualGroundingFilterConfig",
                )
            })?,
            threshold: self.threshold.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "threshold",
                    "threshold was not specified but it is required when building GuardrailContextualGroundingFilterConfig",
                )
            })?,
        })
    }
}
