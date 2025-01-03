// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DeleteSupplementalTaxRegistration`](crate::operation::delete_supplemental_tax_registration::builders::DeleteSupplementalTaxRegistrationFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`authority_id(impl Into<String>)`](crate::operation::delete_supplemental_tax_registration::builders::DeleteSupplementalTaxRegistrationFluentBuilder::authority_id) / [`set_authority_id(Option<String>)`](crate::operation::delete_supplemental_tax_registration::builders::DeleteSupplementalTaxRegistrationFluentBuilder::set_authority_id):<br>required: **true**<br><p>The unique authority Id for the supplemental TRN information that needs to be deleted.</p><br>
    /// - On success, responds with [`DeleteSupplementalTaxRegistrationOutput`](crate::operation::delete_supplemental_tax_registration::DeleteSupplementalTaxRegistrationOutput)
    /// - On failure, responds with [`SdkError<DeleteSupplementalTaxRegistrationError>`](crate::operation::delete_supplemental_tax_registration::DeleteSupplementalTaxRegistrationError)
    pub fn delete_supplemental_tax_registration(
        &self,
    ) -> crate::operation::delete_supplemental_tax_registration::builders::DeleteSupplementalTaxRegistrationFluentBuilder {
        crate::operation::delete_supplemental_tax_registration::builders::DeleteSupplementalTaxRegistrationFluentBuilder::new(self.handle.clone())
    }
}
