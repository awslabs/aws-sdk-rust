// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`RegisterCompute`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`fleet_id(impl Into<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::fleet_id) / [`set_fleet_id(Option<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::set_fleet_id):<br>required: **true**<br><p>A unique identifier for the fleet to register the compute to. You can use either the fleet ID or ARN value.</p><br>
    ///   - [`compute_name(impl Into<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::compute_name) / [`set_compute_name(Option<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::set_compute_name):<br>required: **true**<br><p>A descriptive label for the compute resource.</p><br>
    ///   - [`certificate_path(impl Into<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::certificate_path) / [`set_certificate_path(Option<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::set_certificate_path):<br>required: **false**<br><p>The path to a TLS certificate on your compute resource. Amazon GameLift Servers doesn't validate the path and certificate.</p><br>
    ///   - [`dns_name(impl Into<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::dns_name) / [`set_dns_name(Option<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::set_dns_name):<br>required: **false**<br><p>The DNS name of the compute resource. Amazon GameLift Servers requires either a DNS name or IP address.</p><br>
    ///   - [`ip_address(impl Into<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::ip_address) / [`set_ip_address(Option<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::set_ip_address):<br>required: **false**<br><p>The IP address of the compute resource. Amazon GameLift Servers requires either a DNS name or IP address. When registering an Anywhere fleet, an IP address is required.</p><br>
    ///   - [`location(impl Into<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::location) / [`set_location(Option<String>)`](crate::operation::register_compute::builders::RegisterComputeFluentBuilder::set_location):<br>required: **false**<br><p>The name of a custom location to associate with the compute resource being registered. This parameter is required when registering a compute for an Anywhere fleet.</p><br>
    /// - On success, responds with [`RegisterComputeOutput`](crate::operation::register_compute::RegisterComputeOutput) with field(s):
    ///   - [`compute(Option<Compute>)`](crate::operation::register_compute::RegisterComputeOutput::compute): <p>The details of the compute resource you registered.</p>
    /// - On failure, responds with [`SdkError<RegisterComputeError>`](crate::operation::register_compute::RegisterComputeError)
    pub fn register_compute(&self) -> crate::operation::register_compute::builders::RegisterComputeFluentBuilder {
        crate::operation::register_compute::builders::RegisterComputeFluentBuilder::new(self.handle.clone())
    }
}
