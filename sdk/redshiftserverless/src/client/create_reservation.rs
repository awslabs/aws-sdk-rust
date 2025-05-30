// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`CreateReservation`](crate::operation::create_reservation::builders::CreateReservationFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`capacity(i32)`](crate::operation::create_reservation::builders::CreateReservationFluentBuilder::capacity) / [`set_capacity(Option<i32>)`](crate::operation::create_reservation::builders::CreateReservationFluentBuilder::set_capacity):<br>required: **true**<br><p>The number of Redshift Processing Units (RPUs) to reserve.</p><br>
    ///   - [`offering_id(impl Into<String>)`](crate::operation::create_reservation::builders::CreateReservationFluentBuilder::offering_id) / [`set_offering_id(Option<String>)`](crate::operation::create_reservation::builders::CreateReservationFluentBuilder::set_offering_id):<br>required: **true**<br><p>The ID of the offering associated with the reservation. The offering determines the payment schedule for the reservation.</p><br>
    ///   - [`client_token(impl Into<String>)`](crate::operation::create_reservation::builders::CreateReservationFluentBuilder::client_token) / [`set_client_token(Option<String>)`](crate::operation::create_reservation::builders::CreateReservationFluentBuilder::set_client_token):<br>required: **false**<br><p>A unique, case-sensitive identifier that you provide to ensure the idempotency of the request. If not provided, the Amazon Web Services SDK populates this field. This token must be a valid UUIDv4 value. For more information about idempotency, see <a href="https://aws.amazon.com/builders-library/making-retries-safe-with-idempotent-APIs/"> Making retries safe with idempotent APIs </a>.</p><br>
    /// - On success, responds with [`CreateReservationOutput`](crate::operation::create_reservation::CreateReservationOutput) with field(s):
    ///   - [`reservation(Option<Reservation>)`](crate::operation::create_reservation::CreateReservationOutput::reservation): <p>The reservation object that the <code>CreateReservation</code> action created.</p>
    /// - On failure, responds with [`SdkError<CreateReservationError>`](crate::operation::create_reservation::CreateReservationError)
    pub fn create_reservation(&self) -> crate::operation::create_reservation::builders::CreateReservationFluentBuilder {
        crate::operation::create_reservation::builders::CreateReservationFluentBuilder::new(self.handle.clone())
    }
}
