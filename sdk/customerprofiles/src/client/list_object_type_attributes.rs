// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListObjectTypeAttributes`](crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder::set_next_token):<br>required: **false**<br><p>The pagination token from the previous call.</p><br>
    ///   - [`max_results(i32)`](crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder::set_max_results):<br>required: **false**<br><p>The maximum number of objects returned per page.</p><br>
    ///   - [`domain_name(impl Into<String>)`](crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder::domain_name) / [`set_domain_name(Option<String>)`](crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder::set_domain_name):<br>required: **true**<br><p>The unique identifier of the domain.</p><br>
    ///   - [`object_type_name(impl Into<String>)`](crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder::object_type_name) / [`set_object_type_name(Option<String>)`](crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder::set_object_type_name):<br>required: **true**<br><p>The name of the profile object type.</p><br>
    /// - On success, responds with [`ListObjectTypeAttributesOutput`](crate::operation::list_object_type_attributes::ListObjectTypeAttributesOutput) with field(s):
    ///   - [`items(Option<Vec::<ListObjectTypeAttributeItem>>)`](crate::operation::list_object_type_attributes::ListObjectTypeAttributesOutput::items): <p>The items returned as part of the response.</p>
    ///   - [`next_token(Option<String>)`](crate::operation::list_object_type_attributes::ListObjectTypeAttributesOutput::next_token): <p>The pagination token from the previous call.</p>
    /// - On failure, responds with [`SdkError<ListObjectTypeAttributesError>`](crate::operation::list_object_type_attributes::ListObjectTypeAttributesError)
    pub fn list_object_type_attributes(&self) -> crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder {
        crate::operation::list_object_type_attributes::builders::ListObjectTypeAttributesFluentBuilder::new(self.handle.clone())
    }
}
