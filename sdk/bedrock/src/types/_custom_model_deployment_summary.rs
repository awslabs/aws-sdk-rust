// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>Contains summary information about a custom model deployment, including its ARN, name, status, and associated custom model.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct CustomModelDeploymentSummary {
    /// <p>The Amazon Resource Name (ARN) of the custom model deployment.</p>
    pub custom_model_deployment_arn: ::std::string::String,
    /// <p>The name of the custom model deployment.</p>
    pub custom_model_deployment_name: ::std::string::String,
    /// <p>The Amazon Resource Name (ARN) of the custom model associated with this deployment.</p>
    pub model_arn: ::std::string::String,
    /// <p>The date and time when the custom model deployment was created.</p>
    pub created_at: ::aws_smithy_types::DateTime,
    /// <p>The status of the custom model deployment. Possible values are <code>CREATING</code>, <code>ACTIVE</code>, and <code>FAILED</code>.</p>
    pub status: crate::types::CustomModelDeploymentStatus,
    /// <p>The date and time when the custom model deployment was last modified.</p>
    pub last_updated_at: ::std::option::Option<::aws_smithy_types::DateTime>,
    /// <p>If the deployment status is <code>FAILED</code>, this field contains a message describing the failure reason.</p>
    pub failure_message: ::std::option::Option<::std::string::String>,
}
impl CustomModelDeploymentSummary {
    /// <p>The Amazon Resource Name (ARN) of the custom model deployment.</p>
    pub fn custom_model_deployment_arn(&self) -> &str {
        use std::ops::Deref;
        self.custom_model_deployment_arn.deref()
    }
    /// <p>The name of the custom model deployment.</p>
    pub fn custom_model_deployment_name(&self) -> &str {
        use std::ops::Deref;
        self.custom_model_deployment_name.deref()
    }
    /// <p>The Amazon Resource Name (ARN) of the custom model associated with this deployment.</p>
    pub fn model_arn(&self) -> &str {
        use std::ops::Deref;
        self.model_arn.deref()
    }
    /// <p>The date and time when the custom model deployment was created.</p>
    pub fn created_at(&self) -> &::aws_smithy_types::DateTime {
        &self.created_at
    }
    /// <p>The status of the custom model deployment. Possible values are <code>CREATING</code>, <code>ACTIVE</code>, and <code>FAILED</code>.</p>
    pub fn status(&self) -> &crate::types::CustomModelDeploymentStatus {
        &self.status
    }
    /// <p>The date and time when the custom model deployment was last modified.</p>
    pub fn last_updated_at(&self) -> ::std::option::Option<&::aws_smithy_types::DateTime> {
        self.last_updated_at.as_ref()
    }
    /// <p>If the deployment status is <code>FAILED</code>, this field contains a message describing the failure reason.</p>
    pub fn failure_message(&self) -> ::std::option::Option<&str> {
        self.failure_message.as_deref()
    }
}
impl CustomModelDeploymentSummary {
    /// Creates a new builder-style object to manufacture [`CustomModelDeploymentSummary`](crate::types::CustomModelDeploymentSummary).
    pub fn builder() -> crate::types::builders::CustomModelDeploymentSummaryBuilder {
        crate::types::builders::CustomModelDeploymentSummaryBuilder::default()
    }
}

/// A builder for [`CustomModelDeploymentSummary`](crate::types::CustomModelDeploymentSummary).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct CustomModelDeploymentSummaryBuilder {
    pub(crate) custom_model_deployment_arn: ::std::option::Option<::std::string::String>,
    pub(crate) custom_model_deployment_name: ::std::option::Option<::std::string::String>,
    pub(crate) model_arn: ::std::option::Option<::std::string::String>,
    pub(crate) created_at: ::std::option::Option<::aws_smithy_types::DateTime>,
    pub(crate) status: ::std::option::Option<crate::types::CustomModelDeploymentStatus>,
    pub(crate) last_updated_at: ::std::option::Option<::aws_smithy_types::DateTime>,
    pub(crate) failure_message: ::std::option::Option<::std::string::String>,
}
impl CustomModelDeploymentSummaryBuilder {
    /// <p>The Amazon Resource Name (ARN) of the custom model deployment.</p>
    /// This field is required.
    pub fn custom_model_deployment_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.custom_model_deployment_arn = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the custom model deployment.</p>
    pub fn set_custom_model_deployment_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.custom_model_deployment_arn = input;
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the custom model deployment.</p>
    pub fn get_custom_model_deployment_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.custom_model_deployment_arn
    }
    /// <p>The name of the custom model deployment.</p>
    /// This field is required.
    pub fn custom_model_deployment_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.custom_model_deployment_name = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The name of the custom model deployment.</p>
    pub fn set_custom_model_deployment_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.custom_model_deployment_name = input;
        self
    }
    /// <p>The name of the custom model deployment.</p>
    pub fn get_custom_model_deployment_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.custom_model_deployment_name
    }
    /// <p>The Amazon Resource Name (ARN) of the custom model associated with this deployment.</p>
    /// This field is required.
    pub fn model_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.model_arn = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the custom model associated with this deployment.</p>
    pub fn set_model_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.model_arn = input;
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the custom model associated with this deployment.</p>
    pub fn get_model_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.model_arn
    }
    /// <p>The date and time when the custom model deployment was created.</p>
    /// This field is required.
    pub fn created_at(mut self, input: ::aws_smithy_types::DateTime) -> Self {
        self.created_at = ::std::option::Option::Some(input);
        self
    }
    /// <p>The date and time when the custom model deployment was created.</p>
    pub fn set_created_at(mut self, input: ::std::option::Option<::aws_smithy_types::DateTime>) -> Self {
        self.created_at = input;
        self
    }
    /// <p>The date and time when the custom model deployment was created.</p>
    pub fn get_created_at(&self) -> &::std::option::Option<::aws_smithy_types::DateTime> {
        &self.created_at
    }
    /// <p>The status of the custom model deployment. Possible values are <code>CREATING</code>, <code>ACTIVE</code>, and <code>FAILED</code>.</p>
    /// This field is required.
    pub fn status(mut self, input: crate::types::CustomModelDeploymentStatus) -> Self {
        self.status = ::std::option::Option::Some(input);
        self
    }
    /// <p>The status of the custom model deployment. Possible values are <code>CREATING</code>, <code>ACTIVE</code>, and <code>FAILED</code>.</p>
    pub fn set_status(mut self, input: ::std::option::Option<crate::types::CustomModelDeploymentStatus>) -> Self {
        self.status = input;
        self
    }
    /// <p>The status of the custom model deployment. Possible values are <code>CREATING</code>, <code>ACTIVE</code>, and <code>FAILED</code>.</p>
    pub fn get_status(&self) -> &::std::option::Option<crate::types::CustomModelDeploymentStatus> {
        &self.status
    }
    /// <p>The date and time when the custom model deployment was last modified.</p>
    pub fn last_updated_at(mut self, input: ::aws_smithy_types::DateTime) -> Self {
        self.last_updated_at = ::std::option::Option::Some(input);
        self
    }
    /// <p>The date and time when the custom model deployment was last modified.</p>
    pub fn set_last_updated_at(mut self, input: ::std::option::Option<::aws_smithy_types::DateTime>) -> Self {
        self.last_updated_at = input;
        self
    }
    /// <p>The date and time when the custom model deployment was last modified.</p>
    pub fn get_last_updated_at(&self) -> &::std::option::Option<::aws_smithy_types::DateTime> {
        &self.last_updated_at
    }
    /// <p>If the deployment status is <code>FAILED</code>, this field contains a message describing the failure reason.</p>
    pub fn failure_message(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.failure_message = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>If the deployment status is <code>FAILED</code>, this field contains a message describing the failure reason.</p>
    pub fn set_failure_message(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.failure_message = input;
        self
    }
    /// <p>If the deployment status is <code>FAILED</code>, this field contains a message describing the failure reason.</p>
    pub fn get_failure_message(&self) -> &::std::option::Option<::std::string::String> {
        &self.failure_message
    }
    /// Consumes the builder and constructs a [`CustomModelDeploymentSummary`](crate::types::CustomModelDeploymentSummary).
    /// This method will fail if any of the following fields are not set:
    /// - [`custom_model_deployment_arn`](crate::types::builders::CustomModelDeploymentSummaryBuilder::custom_model_deployment_arn)
    /// - [`custom_model_deployment_name`](crate::types::builders::CustomModelDeploymentSummaryBuilder::custom_model_deployment_name)
    /// - [`model_arn`](crate::types::builders::CustomModelDeploymentSummaryBuilder::model_arn)
    /// - [`created_at`](crate::types::builders::CustomModelDeploymentSummaryBuilder::created_at)
    /// - [`status`](crate::types::builders::CustomModelDeploymentSummaryBuilder::status)
    pub fn build(self) -> ::std::result::Result<crate::types::CustomModelDeploymentSummary, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::CustomModelDeploymentSummary {
            custom_model_deployment_arn: self.custom_model_deployment_arn.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "custom_model_deployment_arn",
                    "custom_model_deployment_arn was not specified but it is required when building CustomModelDeploymentSummary",
                )
            })?,
            custom_model_deployment_name: self.custom_model_deployment_name.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "custom_model_deployment_name",
                    "custom_model_deployment_name was not specified but it is required when building CustomModelDeploymentSummary",
                )
            })?,
            model_arn: self.model_arn.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "model_arn",
                    "model_arn was not specified but it is required when building CustomModelDeploymentSummary",
                )
            })?,
            created_at: self.created_at.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "created_at",
                    "created_at was not specified but it is required when building CustomModelDeploymentSummary",
                )
            })?,
            status: self.status.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "status",
                    "status was not specified but it is required when building CustomModelDeploymentSummary",
                )
            })?,
            last_updated_at: self.last_updated_at,
            failure_message: self.failure_message,
        })
    }
}
