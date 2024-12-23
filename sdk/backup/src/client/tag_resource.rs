// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`TagResource`](crate::operation::tag_resource::builders::TagResourceFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`resource_arn(impl Into<String>)`](crate::operation::tag_resource::builders::TagResourceFluentBuilder::resource_arn) / [`set_resource_arn(Option<String>)`](crate::operation::tag_resource::builders::TagResourceFluentBuilder::set_resource_arn):<br>required: **true**<br><p>An ARN that uniquely identifies a resource. The format of the ARN depends on the type of the tagged resource.</p> <p>ARNs that do not include <code>backup</code> are incompatible with tagging. <code>TagResource</code> and <code>UntagResource</code> with invalid ARNs will result in an error. Acceptable ARN content can include <code>arn:aws:backup:us-east</code>. Invalid ARN content may look like <code>arn:aws:ec2:us-east</code>.</p><br>
    ///   - [`tags(impl Into<String>, impl Into<String>)`](crate::operation::tag_resource::builders::TagResourceFluentBuilder::tags) / [`set_tags(Option<HashMap::<String, String>>)`](crate::operation::tag_resource::builders::TagResourceFluentBuilder::set_tags):<br>required: **true**<br><p>Key-value pairs that are used to help organize your resources. You can assign your own metadata to the resources you create. For clarity, this is the structure to assign tags: <code>\[{"Key":"string","Value":"string"}\]</code>.</p><br>
    /// - On success, responds with [`TagResourceOutput`](crate::operation::tag_resource::TagResourceOutput)
    /// - On failure, responds with [`SdkError<TagResourceError>`](crate::operation::tag_resource::TagResourceError)
    pub fn tag_resource(&self) -> crate::operation::tag_resource::builders::TagResourceFluentBuilder {
        crate::operation::tag_resource::builders::TagResourceFluentBuilder::new(self.handle.clone())
    }
}
