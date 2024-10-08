// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetLinkedWhatsAppBusinessAccountPhoneNumber`](crate::operation::get_linked_whats_app_business_account_phone_number::builders::GetLinkedWhatsAppBusinessAccountPhoneNumberFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`id(impl Into<String>)`](crate::operation::get_linked_whats_app_business_account_phone_number::builders::GetLinkedWhatsAppBusinessAccountPhoneNumberFluentBuilder::id) / [`set_id(Option<String>)`](crate::operation::get_linked_whats_app_business_account_phone_number::builders::GetLinkedWhatsAppBusinessAccountPhoneNumberFluentBuilder::set_id):<br>required: **true**<br><p>The unique identifier of the phone number. Phone number identifiers are formatted as <code>phone-number-id-01234567890123456789012345678901</code>. Use <a href="https://docs.aws.amazon.com/social-messaging/latest/APIReference/API_GetLinkedWhatsAppBusinessAccountPhoneNumber.html">GetLinkedWhatsAppBusinessAccount</a> to find a phone number's id.</p><br>
    /// - On success, responds with [`GetLinkedWhatsAppBusinessAccountPhoneNumberOutput`](crate::operation::get_linked_whats_app_business_account_phone_number::GetLinkedWhatsAppBusinessAccountPhoneNumberOutput) with field(s):
    ///   - [`phone_number(Option<WhatsAppPhoneNumberDetail>)`](crate::operation::get_linked_whats_app_business_account_phone_number::GetLinkedWhatsAppBusinessAccountPhoneNumberOutput::phone_number): <p>The details of your WhatsApp phone number.</p>
    ///   - [`linked_whats_app_business_account_id(Option<String>)`](crate::operation::get_linked_whats_app_business_account_phone_number::GetLinkedWhatsAppBusinessAccountPhoneNumberOutput::linked_whats_app_business_account_id): <p>The WABA identifier linked to the phone number, formatted as <code>waba-01234567890123456789012345678901</code>.</p>
    /// - On failure, responds with [`SdkError<GetLinkedWhatsAppBusinessAccountPhoneNumberError>`](crate::operation::get_linked_whats_app_business_account_phone_number::GetLinkedWhatsAppBusinessAccountPhoneNumberError)
    pub fn get_linked_whats_app_business_account_phone_number(
        &self,
    ) -> crate::operation::get_linked_whats_app_business_account_phone_number::builders::GetLinkedWhatsAppBusinessAccountPhoneNumberFluentBuilder
    {
        crate::operation::get_linked_whats_app_business_account_phone_number::builders::GetLinkedWhatsAppBusinessAccountPhoneNumberFluentBuilder::new(
            self.handle.clone(),
        )
    }
}
