// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`AttachVolume`](crate::operation::attach_volume::builders::AttachVolumeFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`device(impl Into<String>)`](crate::operation::attach_volume::builders::AttachVolumeFluentBuilder::device) / [`set_device(Option<String>)`](crate::operation::attach_volume::builders::AttachVolumeFluentBuilder::set_device):<br>required: **true**<br><p>The device name (for example, <code>/dev/sdh</code> or <code>xvdh</code>).</p><br>
    ///   - [`instance_id(impl Into<String>)`](crate::operation::attach_volume::builders::AttachVolumeFluentBuilder::instance_id) / [`set_instance_id(Option<String>)`](crate::operation::attach_volume::builders::AttachVolumeFluentBuilder::set_instance_id):<br>required: **true**<br><p>The ID of the instance.</p><br>
    ///   - [`volume_id(impl Into<String>)`](crate::operation::attach_volume::builders::AttachVolumeFluentBuilder::volume_id) / [`set_volume_id(Option<String>)`](crate::operation::attach_volume::builders::AttachVolumeFluentBuilder::set_volume_id):<br>required: **true**<br><p>The ID of the EBS volume. The volume and instance must be within the same Availability Zone.</p><br>
    ///   - [`dry_run(bool)`](crate::operation::attach_volume::builders::AttachVolumeFluentBuilder::dry_run) / [`set_dry_run(Option<bool>)`](crate::operation::attach_volume::builders::AttachVolumeFluentBuilder::set_dry_run):<br>required: **false**<br><p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p><br>
    /// - On success, responds with [`AttachVolumeOutput`](crate::operation::attach_volume::AttachVolumeOutput) with field(s):
    ///   - [`delete_on_termination(Option<bool>)`](crate::operation::attach_volume::AttachVolumeOutput::delete_on_termination): <p>Indicates whether the EBS volume is deleted on instance termination.</p>
    ///   - [`associated_resource(Option<String>)`](crate::operation::attach_volume::AttachVolumeOutput::associated_resource): <p>The ARN of the Amazon Web Services-managed resource to which the volume is attached.</p>
    ///   - [`instance_owning_service(Option<String>)`](crate::operation::attach_volume::AttachVolumeOutput::instance_owning_service): <p>The service principal of the Amazon Web Services service that owns the underlying resource to which the volume is attached.</p> <p>This parameter is returned only for volumes that are attached to Amazon Web Services-managed resources.</p>
    ///   - [`volume_id(Option<String>)`](crate::operation::attach_volume::AttachVolumeOutput::volume_id): <p>The ID of the volume.</p>
    ///   - [`instance_id(Option<String>)`](crate::operation::attach_volume::AttachVolumeOutput::instance_id): <p>The ID of the instance.</p> <p>If the volume is attached to an Amazon Web Services-managed resource, this parameter returns <code>null</code>.</p>
    ///   - [`device(Option<String>)`](crate::operation::attach_volume::AttachVolumeOutput::device): <p>The device name.</p> <p>If the volume is attached to an Amazon Web Services-managed resource, this parameter returns <code>null</code>.</p>
    ///   - [`state(Option<VolumeAttachmentState>)`](crate::operation::attach_volume::AttachVolumeOutput::state): <p>The attachment state of the volume.</p>
    ///   - [`attach_time(Option<DateTime>)`](crate::operation::attach_volume::AttachVolumeOutput::attach_time): <p>The time stamp when the attachment initiated.</p>
    /// - On failure, responds with [`SdkError<AttachVolumeError>`](crate::operation::attach_volume::AttachVolumeError)
    pub fn attach_volume(&self) -> crate::operation::attach_volume::builders::AttachVolumeFluentBuilder {
        crate::operation::attach_volume::builders::AttachVolumeFluentBuilder::new(self.handle.clone())
    }
}
