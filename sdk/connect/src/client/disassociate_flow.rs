// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DisassociateFlow`](crate::operation::disassociate_flow::builders::DisassociateFlowFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`instance_id(impl Into<String>)`](crate::operation::disassociate_flow::builders::DisassociateFlowFluentBuilder::instance_id) / [`set_instance_id(Option<String>)`](crate::operation::disassociate_flow::builders::DisassociateFlowFluentBuilder::set_instance_id):<br>required: **true**<br><p>The identifier of the Amazon Connect instance. You can <a href="https://docs.aws.amazon.com/connect/latest/adminguide/find-instance-arn.html">find the instance ID</a> in the Amazon Resource Name (ARN) of the instance.</p><br>
    ///   - [`resource_id(impl Into<String>)`](crate::operation::disassociate_flow::builders::DisassociateFlowFluentBuilder::resource_id) / [`set_resource_id(Option<String>)`](crate::operation::disassociate_flow::builders::DisassociateFlowFluentBuilder::set_resource_id):<br>required: **true**<br><p>The identifier of the resource.</p> <ul>  <li>   <p>Amazon Web Services End User Messaging SMS phone number ARN when using <code>SMS_PHONE_NUMBER</code></p></li>  <li>   <p>Amazon Web Services End User Messaging Social phone number ARN when using <code>WHATSAPP_MESSAGING_PHONE_NUMBER</code></p></li> </ul><br>
    ///   - [`resource_type(FlowAssociationResourceType)`](crate::operation::disassociate_flow::builders::DisassociateFlowFluentBuilder::resource_type) / [`set_resource_type(Option<FlowAssociationResourceType>)`](crate::operation::disassociate_flow::builders::DisassociateFlowFluentBuilder::set_resource_type):<br>required: **true**<br><p>A valid resource type.</p><br>
    /// - On success, responds with [`DisassociateFlowOutput`](crate::operation::disassociate_flow::DisassociateFlowOutput)
    /// - On failure, responds with [`SdkError<DisassociateFlowError>`](crate::operation::disassociate_flow::DisassociateFlowError)
    pub fn disassociate_flow(&self) -> crate::operation::disassociate_flow::builders::DisassociateFlowFluentBuilder {
        crate::operation::disassociate_flow::builders::DisassociateFlowFluentBuilder::new(self.handle.clone())
    }
}
