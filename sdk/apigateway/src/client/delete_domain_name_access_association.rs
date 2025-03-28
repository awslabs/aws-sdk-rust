// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DeleteDomainNameAccessAssociation`](crate::operation::delete_domain_name_access_association::builders::DeleteDomainNameAccessAssociationFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`domain_name_access_association_arn(impl Into<String>)`](crate::operation::delete_domain_name_access_association::builders::DeleteDomainNameAccessAssociationFluentBuilder::domain_name_access_association_arn) / [`set_domain_name_access_association_arn(Option<String>)`](crate::operation::delete_domain_name_access_association::builders::DeleteDomainNameAccessAssociationFluentBuilder::set_domain_name_access_association_arn):<br>required: **true**<br><p>The ARN of the domain name access association resource.</p><br>
    /// - On success, responds with [`DeleteDomainNameAccessAssociationOutput`](crate::operation::delete_domain_name_access_association::DeleteDomainNameAccessAssociationOutput)
    /// - On failure, responds with [`SdkError<DeleteDomainNameAccessAssociationError>`](crate::operation::delete_domain_name_access_association::DeleteDomainNameAccessAssociationError)
    pub fn delete_domain_name_access_association(
        &self,
    ) -> crate::operation::delete_domain_name_access_association::builders::DeleteDomainNameAccessAssociationFluentBuilder {
        crate::operation::delete_domain_name_access_association::builders::DeleteDomainNameAccessAssociationFluentBuilder::new(self.handle.clone())
    }
}
