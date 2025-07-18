// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>A complex structure for a managed resource in an Amazon Web Services account with information about zonal shifts and autoshifts.</p>
/// <p>You can start a zonal shift in ARC for a managed resource to temporarily move traffic for the resource away from an Availability Zone in an Amazon Web Services Region. You can also configure zonal autoshift for a managed resource.</p><note>
/// <p>At this time, managed resources are Amazon EC2 Auto Scaling groups, Amazon Elastic Kubernetes Service, Network Load Balancers, and Application Load Balancer.</p>
/// </note>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ManagedResourceSummary {
    /// <p>The Amazon Resource Name (ARN) for the managed resource.</p>
    pub arn: ::std::option::Option<::std::string::String>,
    /// <p>The name of the managed resource.</p>
    pub name: ::std::option::Option<::std::string::String>,
    /// <p>The Availability Zones that a resource is deployed in.</p>
    pub availability_zones: ::std::vec::Vec<::std::string::String>,
    /// <p>A collection of key-value pairs that indicate whether resources are active in Availability Zones or not. The key name is the Availability Zone where the resource is deployed. The value is 1 or 0.</p>
    pub applied_weights: ::std::option::Option<::std::collections::HashMap<::std::string::String, f32>>,
    /// <p>An array of the zonal shifts for a resource.</p>
    pub zonal_shifts: ::std::option::Option<::std::vec::Vec<crate::types::ZonalShiftInResource>>,
    /// <p>An array of the autoshifts that have been completed for a resource.</p>
    pub autoshifts: ::std::option::Option<::std::vec::Vec<crate::types::AutoshiftInResource>>,
    /// <p>The status of autoshift for a resource. When you configure zonal autoshift for a resource, you can set the value of the status to <code>ENABLED</code> or <code>DISABLED</code>.</p>
    pub zonal_autoshift_status: ::std::option::Option<crate::types::ZonalAutoshiftStatus>,
    /// <p>This status tracks whether a practice run configuration exists for a resource. When you configure a practice run for a resource so that a practice run configuration exists, ARC sets this value to <code>ENABLED</code>. If a you have not configured a practice run for the resource, or delete a practice run configuration, ARC sets the value to <code>DISABLED</code>.</p>
    /// <p>ARC updates this status; you can't set a practice run status to <code>ENABLED</code> or <code>DISABLED</code>.</p>
    pub practice_run_status: ::std::option::Option<crate::types::ZonalAutoshiftStatus>,
}
impl ManagedResourceSummary {
    /// <p>The Amazon Resource Name (ARN) for the managed resource.</p>
    pub fn arn(&self) -> ::std::option::Option<&str> {
        self.arn.as_deref()
    }
    /// <p>The name of the managed resource.</p>
    pub fn name(&self) -> ::std::option::Option<&str> {
        self.name.as_deref()
    }
    /// <p>The Availability Zones that a resource is deployed in.</p>
    pub fn availability_zones(&self) -> &[::std::string::String] {
        use std::ops::Deref;
        self.availability_zones.deref()
    }
    /// <p>A collection of key-value pairs that indicate whether resources are active in Availability Zones or not. The key name is the Availability Zone where the resource is deployed. The value is 1 or 0.</p>
    pub fn applied_weights(&self) -> ::std::option::Option<&::std::collections::HashMap<::std::string::String, f32>> {
        self.applied_weights.as_ref()
    }
    /// <p>An array of the zonal shifts for a resource.</p>
    ///
    /// If no value was sent for this field, a default will be set. If you want to determine if no value was sent, use `.zonal_shifts.is_none()`.
    pub fn zonal_shifts(&self) -> &[crate::types::ZonalShiftInResource] {
        self.zonal_shifts.as_deref().unwrap_or_default()
    }
    /// <p>An array of the autoshifts that have been completed for a resource.</p>
    ///
    /// If no value was sent for this field, a default will be set. If you want to determine if no value was sent, use `.autoshifts.is_none()`.
    pub fn autoshifts(&self) -> &[crate::types::AutoshiftInResource] {
        self.autoshifts.as_deref().unwrap_or_default()
    }
    /// <p>The status of autoshift for a resource. When you configure zonal autoshift for a resource, you can set the value of the status to <code>ENABLED</code> or <code>DISABLED</code>.</p>
    pub fn zonal_autoshift_status(&self) -> ::std::option::Option<&crate::types::ZonalAutoshiftStatus> {
        self.zonal_autoshift_status.as_ref()
    }
    /// <p>This status tracks whether a practice run configuration exists for a resource. When you configure a practice run for a resource so that a practice run configuration exists, ARC sets this value to <code>ENABLED</code>. If a you have not configured a practice run for the resource, or delete a practice run configuration, ARC sets the value to <code>DISABLED</code>.</p>
    /// <p>ARC updates this status; you can't set a practice run status to <code>ENABLED</code> or <code>DISABLED</code>.</p>
    pub fn practice_run_status(&self) -> ::std::option::Option<&crate::types::ZonalAutoshiftStatus> {
        self.practice_run_status.as_ref()
    }
}
impl ManagedResourceSummary {
    /// Creates a new builder-style object to manufacture [`ManagedResourceSummary`](crate::types::ManagedResourceSummary).
    pub fn builder() -> crate::types::builders::ManagedResourceSummaryBuilder {
        crate::types::builders::ManagedResourceSummaryBuilder::default()
    }
}

/// A builder for [`ManagedResourceSummary`](crate::types::ManagedResourceSummary).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct ManagedResourceSummaryBuilder {
    pub(crate) arn: ::std::option::Option<::std::string::String>,
    pub(crate) name: ::std::option::Option<::std::string::String>,
    pub(crate) availability_zones: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
    pub(crate) applied_weights: ::std::option::Option<::std::collections::HashMap<::std::string::String, f32>>,
    pub(crate) zonal_shifts: ::std::option::Option<::std::vec::Vec<crate::types::ZonalShiftInResource>>,
    pub(crate) autoshifts: ::std::option::Option<::std::vec::Vec<crate::types::AutoshiftInResource>>,
    pub(crate) zonal_autoshift_status: ::std::option::Option<crate::types::ZonalAutoshiftStatus>,
    pub(crate) practice_run_status: ::std::option::Option<crate::types::ZonalAutoshiftStatus>,
}
impl ManagedResourceSummaryBuilder {
    /// <p>The Amazon Resource Name (ARN) for the managed resource.</p>
    pub fn arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.arn = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) for the managed resource.</p>
    pub fn set_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.arn = input;
        self
    }
    /// <p>The Amazon Resource Name (ARN) for the managed resource.</p>
    pub fn get_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.arn
    }
    /// <p>The name of the managed resource.</p>
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.name = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The name of the managed resource.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.name = input;
        self
    }
    /// <p>The name of the managed resource.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.name
    }
    /// Appends an item to `availability_zones`.
    ///
    /// To override the contents of this collection use [`set_availability_zones`](Self::set_availability_zones).
    ///
    /// <p>The Availability Zones that a resource is deployed in.</p>
    pub fn availability_zones(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        let mut v = self.availability_zones.unwrap_or_default();
        v.push(input.into());
        self.availability_zones = ::std::option::Option::Some(v);
        self
    }
    /// <p>The Availability Zones that a resource is deployed in.</p>
    pub fn set_availability_zones(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.availability_zones = input;
        self
    }
    /// <p>The Availability Zones that a resource is deployed in.</p>
    pub fn get_availability_zones(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        &self.availability_zones
    }
    /// Adds a key-value pair to `applied_weights`.
    ///
    /// To override the contents of this collection use [`set_applied_weights`](Self::set_applied_weights).
    ///
    /// <p>A collection of key-value pairs that indicate whether resources are active in Availability Zones or not. The key name is the Availability Zone where the resource is deployed. The value is 1 or 0.</p>
    pub fn applied_weights(mut self, k: impl ::std::convert::Into<::std::string::String>, v: f32) -> Self {
        let mut hash_map = self.applied_weights.unwrap_or_default();
        hash_map.insert(k.into(), v);
        self.applied_weights = ::std::option::Option::Some(hash_map);
        self
    }
    /// <p>A collection of key-value pairs that indicate whether resources are active in Availability Zones or not. The key name is the Availability Zone where the resource is deployed. The value is 1 or 0.</p>
    pub fn set_applied_weights(mut self, input: ::std::option::Option<::std::collections::HashMap<::std::string::String, f32>>) -> Self {
        self.applied_weights = input;
        self
    }
    /// <p>A collection of key-value pairs that indicate whether resources are active in Availability Zones or not. The key name is the Availability Zone where the resource is deployed. The value is 1 or 0.</p>
    pub fn get_applied_weights(&self) -> &::std::option::Option<::std::collections::HashMap<::std::string::String, f32>> {
        &self.applied_weights
    }
    /// Appends an item to `zonal_shifts`.
    ///
    /// To override the contents of this collection use [`set_zonal_shifts`](Self::set_zonal_shifts).
    ///
    /// <p>An array of the zonal shifts for a resource.</p>
    pub fn zonal_shifts(mut self, input: crate::types::ZonalShiftInResource) -> Self {
        let mut v = self.zonal_shifts.unwrap_or_default();
        v.push(input);
        self.zonal_shifts = ::std::option::Option::Some(v);
        self
    }
    /// <p>An array of the zonal shifts for a resource.</p>
    pub fn set_zonal_shifts(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::ZonalShiftInResource>>) -> Self {
        self.zonal_shifts = input;
        self
    }
    /// <p>An array of the zonal shifts for a resource.</p>
    pub fn get_zonal_shifts(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::ZonalShiftInResource>> {
        &self.zonal_shifts
    }
    /// Appends an item to `autoshifts`.
    ///
    /// To override the contents of this collection use [`set_autoshifts`](Self::set_autoshifts).
    ///
    /// <p>An array of the autoshifts that have been completed for a resource.</p>
    pub fn autoshifts(mut self, input: crate::types::AutoshiftInResource) -> Self {
        let mut v = self.autoshifts.unwrap_or_default();
        v.push(input);
        self.autoshifts = ::std::option::Option::Some(v);
        self
    }
    /// <p>An array of the autoshifts that have been completed for a resource.</p>
    pub fn set_autoshifts(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::AutoshiftInResource>>) -> Self {
        self.autoshifts = input;
        self
    }
    /// <p>An array of the autoshifts that have been completed for a resource.</p>
    pub fn get_autoshifts(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::AutoshiftInResource>> {
        &self.autoshifts
    }
    /// <p>The status of autoshift for a resource. When you configure zonal autoshift for a resource, you can set the value of the status to <code>ENABLED</code> or <code>DISABLED</code>.</p>
    pub fn zonal_autoshift_status(mut self, input: crate::types::ZonalAutoshiftStatus) -> Self {
        self.zonal_autoshift_status = ::std::option::Option::Some(input);
        self
    }
    /// <p>The status of autoshift for a resource. When you configure zonal autoshift for a resource, you can set the value of the status to <code>ENABLED</code> or <code>DISABLED</code>.</p>
    pub fn set_zonal_autoshift_status(mut self, input: ::std::option::Option<crate::types::ZonalAutoshiftStatus>) -> Self {
        self.zonal_autoshift_status = input;
        self
    }
    /// <p>The status of autoshift for a resource. When you configure zonal autoshift for a resource, you can set the value of the status to <code>ENABLED</code> or <code>DISABLED</code>.</p>
    pub fn get_zonal_autoshift_status(&self) -> &::std::option::Option<crate::types::ZonalAutoshiftStatus> {
        &self.zonal_autoshift_status
    }
    /// <p>This status tracks whether a practice run configuration exists for a resource. When you configure a practice run for a resource so that a practice run configuration exists, ARC sets this value to <code>ENABLED</code>. If a you have not configured a practice run for the resource, or delete a practice run configuration, ARC sets the value to <code>DISABLED</code>.</p>
    /// <p>ARC updates this status; you can't set a practice run status to <code>ENABLED</code> or <code>DISABLED</code>.</p>
    pub fn practice_run_status(mut self, input: crate::types::ZonalAutoshiftStatus) -> Self {
        self.practice_run_status = ::std::option::Option::Some(input);
        self
    }
    /// <p>This status tracks whether a practice run configuration exists for a resource. When you configure a practice run for a resource so that a practice run configuration exists, ARC sets this value to <code>ENABLED</code>. If a you have not configured a practice run for the resource, or delete a practice run configuration, ARC sets the value to <code>DISABLED</code>.</p>
    /// <p>ARC updates this status; you can't set a practice run status to <code>ENABLED</code> or <code>DISABLED</code>.</p>
    pub fn set_practice_run_status(mut self, input: ::std::option::Option<crate::types::ZonalAutoshiftStatus>) -> Self {
        self.practice_run_status = input;
        self
    }
    /// <p>This status tracks whether a practice run configuration exists for a resource. When you configure a practice run for a resource so that a practice run configuration exists, ARC sets this value to <code>ENABLED</code>. If a you have not configured a practice run for the resource, or delete a practice run configuration, ARC sets the value to <code>DISABLED</code>.</p>
    /// <p>ARC updates this status; you can't set a practice run status to <code>ENABLED</code> or <code>DISABLED</code>.</p>
    pub fn get_practice_run_status(&self) -> &::std::option::Option<crate::types::ZonalAutoshiftStatus> {
        &self.practice_run_status
    }
    /// Consumes the builder and constructs a [`ManagedResourceSummary`](crate::types::ManagedResourceSummary).
    /// This method will fail if any of the following fields are not set:
    /// - [`availability_zones`](crate::types::builders::ManagedResourceSummaryBuilder::availability_zones)
    pub fn build(self) -> ::std::result::Result<crate::types::ManagedResourceSummary, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::ManagedResourceSummary {
            arn: self.arn,
            name: self.name,
            availability_zones: self.availability_zones.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "availability_zones",
                    "availability_zones was not specified but it is required when building ManagedResourceSummary",
                )
            })?,
            applied_weights: self.applied_weights,
            zonal_shifts: self.zonal_shifts,
            autoshifts: self.autoshifts,
            zonal_autoshift_status: self.zonal_autoshift_status,
            practice_run_status: self.practice_run_status,
        })
    }
}
