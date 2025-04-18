// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`UpdateWebApp`](crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`web_app_id(impl Into<String>)`](crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder::web_app_id) / [`set_web_app_id(Option<String>)`](crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder::set_web_app_id):<br>required: **true**<br><p>Provide the identifier of the web app that you are updating.</p><br>
    ///   - [`identity_provider_details(UpdateWebAppIdentityProviderDetails)`](crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder::identity_provider_details) / [`set_identity_provider_details(Option<UpdateWebAppIdentityProviderDetails>)`](crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder::set_identity_provider_details):<br>required: **false**<br><p>Provide updated identity provider values in a <code>WebAppIdentityProviderDetails</code> object.</p><br>
    ///   - [`access_endpoint(impl Into<String>)`](crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder::access_endpoint) / [`set_access_endpoint(Option<String>)`](crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder::set_access_endpoint):<br>required: **false**<br><p>The <code>AccessEndpoint</code> is the URL that you provide to your users for them to interact with the Transfer Family web app. You can specify a custom URL or use the default value.</p><br>
    ///   - [`web_app_units(WebAppUnits)`](crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder::web_app_units) / [`set_web_app_units(Option<WebAppUnits>)`](crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder::set_web_app_units):<br>required: **false**<br><p>A union that contains the value for number of concurrent connections or the user sessions on your web app.</p><br>
    /// - On success, responds with [`UpdateWebAppOutput`](crate::operation::update_web_app::UpdateWebAppOutput) with field(s):
    ///   - [`web_app_id(String)`](crate::operation::update_web_app::UpdateWebAppOutput::web_app_id): <p>Returns the unique identifier for the web app being updated.</p>
    /// - On failure, responds with [`SdkError<UpdateWebAppError>`](crate::operation::update_web_app::UpdateWebAppError)
    pub fn update_web_app(&self) -> crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder {
        crate::operation::update_web_app::builders::UpdateWebAppFluentBuilder::new(self.handle.clone())
    }
}
