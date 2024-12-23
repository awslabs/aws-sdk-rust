// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DescribeWebAppCustomization`](crate::operation::describe_web_app_customization::builders::DescribeWebAppCustomizationFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`web_app_id(impl Into<String>)`](crate::operation::describe_web_app_customization::builders::DescribeWebAppCustomizationFluentBuilder::web_app_id) / [`set_web_app_id(Option<String>)`](crate::operation::describe_web_app_customization::builders::DescribeWebAppCustomizationFluentBuilder::set_web_app_id):<br>required: **true**<br><p>Provide the unique identifier for the web app.</p><br>
    /// - On success, responds with [`DescribeWebAppCustomizationOutput`](crate::operation::describe_web_app_customization::DescribeWebAppCustomizationOutput) with field(s):
    ///   - [`web_app_customization(Option<DescribedWebAppCustomization>)`](crate::operation::describe_web_app_customization::DescribeWebAppCustomizationOutput::web_app_customization): <p>Returns a structure that contains the details of the web app customizations.</p>
    /// - On failure, responds with [`SdkError<DescribeWebAppCustomizationError>`](crate::operation::describe_web_app_customization::DescribeWebAppCustomizationError)
    pub fn describe_web_app_customization(
        &self,
    ) -> crate::operation::describe_web_app_customization::builders::DescribeWebAppCustomizationFluentBuilder {
        crate::operation::describe_web_app_customization::builders::DescribeWebAppCustomizationFluentBuilder::new(self.handle.clone())
    }
}
