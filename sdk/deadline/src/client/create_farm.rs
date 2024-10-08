// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`CreateFarm`](crate::operation::create_farm::builders::CreateFarmFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`client_token(impl Into<String>)`](crate::operation::create_farm::builders::CreateFarmFluentBuilder::client_token) / [`set_client_token(Option<String>)`](crate::operation::create_farm::builders::CreateFarmFluentBuilder::set_client_token):<br>required: **false**<br><p>The unique token which the server uses to recognize retries of the same request.</p><br>
    ///   - [`display_name(impl Into<String>)`](crate::operation::create_farm::builders::CreateFarmFluentBuilder::display_name) / [`set_display_name(Option<String>)`](crate::operation::create_farm::builders::CreateFarmFluentBuilder::set_display_name):<br>required: **true**<br><p>The display name of the farm.</p><important>  <p>This field can store any content. Escape or encode this content before displaying it on a webpage or any other system that might interpret the content of this field.</p> </important><br>
    ///   - [`description(impl Into<String>)`](crate::operation::create_farm::builders::CreateFarmFluentBuilder::description) / [`set_description(Option<String>)`](crate::operation::create_farm::builders::CreateFarmFluentBuilder::set_description):<br>required: **false**<br><p>The description of the farm.</p><important>  <p>This field can store any content. Escape or encode this content before displaying it on a webpage or any other system that might interpret the content of this field.</p> </important><br>
    ///   - [`kms_key_arn(impl Into<String>)`](crate::operation::create_farm::builders::CreateFarmFluentBuilder::kms_key_arn) / [`set_kms_key_arn(Option<String>)`](crate::operation::create_farm::builders::CreateFarmFluentBuilder::set_kms_key_arn):<br>required: **false**<br><p>The ARN of the KMS key to use on the farm.</p><br>
    ///   - [`tags(impl Into<String>, impl Into<String>)`](crate::operation::create_farm::builders::CreateFarmFluentBuilder::tags) / [`set_tags(Option<HashMap::<String, String>>)`](crate::operation::create_farm::builders::CreateFarmFluentBuilder::set_tags):<br>required: **false**<br><p>The tags to add to your farm. Each tag consists of a tag key and a tag value. Tag keys and values are both required, but tag values can be empty strings.</p><br>
    /// - On success, responds with [`CreateFarmOutput`](crate::operation::create_farm::CreateFarmOutput) with field(s):
    ///   - [`farm_id(String)`](crate::operation::create_farm::CreateFarmOutput::farm_id): <p>The farm ID.</p>
    /// - On failure, responds with [`SdkError<CreateFarmError>`](crate::operation::create_farm::CreateFarmError)
    pub fn create_farm(&self) -> crate::operation::create_farm::builders::CreateFarmFluentBuilder {
        crate::operation::create_farm::builders::CreateFarmFluentBuilder::new(self.handle.clone())
    }
}
