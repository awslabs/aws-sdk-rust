// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`UpdateResourceServer`](crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`user_pool_id(impl Into<String>)`](crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder::user_pool_id) / [`set_user_pool_id(Option<String>)`](crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder::set_user_pool_id):<br>required: **true**<br><p>The ID of the user pool that contains the resource server that you want to update.</p><br>
    ///   - [`identifier(impl Into<String>)`](crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder::identifier) / [`set_identifier(Option<String>)`](crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder::set_identifier):<br>required: **true**<br><p>A unique resource server identifier for the resource server. The identifier can be an API friendly name like <code>solar-system-data</code>. You can also set an API URL like <code>https://solar-system-data-api.example.com</code> as your identifier.</p> <p>Amazon Cognito represents scopes in the access token in the format <code>$resource-server-identifier/$scope</code>. Longer scope-identifier strings increase the size of your access tokens.</p><br>
    ///   - [`name(impl Into<String>)`](crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder::name) / [`set_name(Option<String>)`](crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder::set_name):<br>required: **true**<br><p>The updated name of the resource server.</p><br>
    ///   - [`scopes(ResourceServerScopeType)`](crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder::scopes) / [`set_scopes(Option<Vec::<ResourceServerScopeType>>)`](crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder::set_scopes):<br>required: **false**<br><p>An array of updated custom scope names and descriptions that you want to associate with your resource server.</p><br>
    /// - On success, responds with [`UpdateResourceServerOutput`](crate::operation::update_resource_server::UpdateResourceServerOutput) with field(s):
    ///   - [`resource_server(Option<ResourceServerType>)`](crate::operation::update_resource_server::UpdateResourceServerOutput::resource_server): <p>The updated details of the requested resource server.</p>
    /// - On failure, responds with [`SdkError<UpdateResourceServerError>`](crate::operation::update_resource_server::UpdateResourceServerError)
    pub fn update_resource_server(&self) -> crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder {
        crate::operation::update_resource_server::builders::UpdateResourceServerFluentBuilder::new(self.handle.clone())
    }
}
