// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetUserAuthFactors`](crate::operation::get_user_auth_factors::builders::GetUserAuthFactorsFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`access_token(impl Into<String>)`](crate::operation::get_user_auth_factors::builders::GetUserAuthFactorsFluentBuilder::access_token) / [`set_access_token(Option<String>)`](crate::operation::get_user_auth_factors::builders::GetUserAuthFactorsFluentBuilder::set_access_token):<br>required: **true**<br><p>A valid access token that Amazon Cognito issued to the currently signed-in user. Must include a scope claim for <code>aws.cognito.signin.user.admin</code>.</p><br>
    /// - On success, responds with [`GetUserAuthFactorsOutput`](crate::operation::get_user_auth_factors::GetUserAuthFactorsOutput) with field(s):
    ///   - [`username(String)`](crate::operation::get_user_auth_factors::GetUserAuthFactorsOutput::username): <p>The name of the user who is eligible for the authentication factors in the response.</p>
    ///   - [`preferred_mfa_setting(Option<String>)`](crate::operation::get_user_auth_factors::GetUserAuthFactorsOutput::preferred_mfa_setting): <p>The challenge method that Amazon Cognito returns to the user in response to sign-in requests. Users can prefer SMS message, email message, or TOTP MFA.</p>
    ///   - [`user_mfa_setting_list(Option<Vec::<String>>)`](crate::operation::get_user_auth_factors::GetUserAuthFactorsOutput::user_mfa_setting_list): <p>The MFA options that are activated for the user. The possible values in this list are <code>SMS_MFA</code>, <code>EMAIL_OTP</code>, and <code>SOFTWARE_TOKEN_MFA</code>.</p>
    ///   - [`configured_user_auth_factors(Option<Vec::<AuthFactorType>>)`](crate::operation::get_user_auth_factors::GetUserAuthFactorsOutput::configured_user_auth_factors): <p>The authentication types that are available to the user with <code>USER_AUTH</code> sign-in, for example <code>\["PASSWORD", "WEB_AUTHN"\]</code>.</p>
    /// - On failure, responds with [`SdkError<GetUserAuthFactorsError>`](crate::operation::get_user_auth_factors::GetUserAuthFactorsError)
    pub fn get_user_auth_factors(&self) -> crate::operation::get_user_auth_factors::builders::GetUserAuthFactorsFluentBuilder {
        crate::operation::get_user_auth_factors::builders::GetUserAuthFactorsFluentBuilder::new(self.handle.clone())
    }
}
