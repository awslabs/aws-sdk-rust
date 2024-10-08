// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DisassociateCapacityReservationBillingOwner`](crate::operation::disassociate_capacity_reservation_billing_owner::builders::DisassociateCapacityReservationBillingOwnerFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`dry_run(bool)`](crate::operation::disassociate_capacity_reservation_billing_owner::builders::DisassociateCapacityReservationBillingOwnerFluentBuilder::dry_run) / [`set_dry_run(Option<bool>)`](crate::operation::disassociate_capacity_reservation_billing_owner::builders::DisassociateCapacityReservationBillingOwnerFluentBuilder::set_dry_run):<br>required: **false**<br><p>Checks whether you have the required permissions for the action, without actually making the request, and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p><br>
    ///   - [`capacity_reservation_id(impl Into<String>)`](crate::operation::disassociate_capacity_reservation_billing_owner::builders::DisassociateCapacityReservationBillingOwnerFluentBuilder::capacity_reservation_id) / [`set_capacity_reservation_id(Option<String>)`](crate::operation::disassociate_capacity_reservation_billing_owner::builders::DisassociateCapacityReservationBillingOwnerFluentBuilder::set_capacity_reservation_id):<br>required: **true**<br><p>The ID of the Capacity Reservation.</p><br>
    ///   - [`unused_reservation_billing_owner_id(impl Into<String>)`](crate::operation::disassociate_capacity_reservation_billing_owner::builders::DisassociateCapacityReservationBillingOwnerFluentBuilder::unused_reservation_billing_owner_id) / [`set_unused_reservation_billing_owner_id(Option<String>)`](crate::operation::disassociate_capacity_reservation_billing_owner::builders::DisassociateCapacityReservationBillingOwnerFluentBuilder::set_unused_reservation_billing_owner_id):<br>required: **true**<br><p>The ID of the consumer account to which the request was sent.</p><br>
    /// - On success, responds with [`DisassociateCapacityReservationBillingOwnerOutput`](crate::operation::disassociate_capacity_reservation_billing_owner::DisassociateCapacityReservationBillingOwnerOutput) with field(s):
    ///   - [`r#return(Option<bool>)`](crate::operation::disassociate_capacity_reservation_billing_owner::DisassociateCapacityReservationBillingOwnerOutput::return): <p>Returns <code>true</code> if the request succeeds; otherwise, it returns an error.</p>
    /// - On failure, responds with [`SdkError<DisassociateCapacityReservationBillingOwnerError>`](crate::operation::disassociate_capacity_reservation_billing_owner::DisassociateCapacityReservationBillingOwnerError)
    pub fn disassociate_capacity_reservation_billing_owner(
        &self,
    ) -> crate::operation::disassociate_capacity_reservation_billing_owner::builders::DisassociateCapacityReservationBillingOwnerFluentBuilder {
        crate::operation::disassociate_capacity_reservation_billing_owner::builders::DisassociateCapacityReservationBillingOwnerFluentBuilder::new(
            self.handle.clone(),
        )
    }
}
