// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetLFTagExpression`](crate::operation::get_lf_tag_expression::builders::GetLFTagExpressionFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`name(impl Into<String>)`](crate::operation::get_lf_tag_expression::builders::GetLFTagExpressionFluentBuilder::name) / [`set_name(Option<String>)`](crate::operation::get_lf_tag_expression::builders::GetLFTagExpressionFluentBuilder::set_name):<br>required: **true**<br><p>The name for the LF-Tag expression</p><br>
    ///   - [`catalog_id(impl Into<String>)`](crate::operation::get_lf_tag_expression::builders::GetLFTagExpressionFluentBuilder::catalog_id) / [`set_catalog_id(Option<String>)`](crate::operation::get_lf_tag_expression::builders::GetLFTagExpressionFluentBuilder::set_catalog_id):<br>required: **false**<br><p>The identifier for the Data Catalog. By default, the account ID.</p><br>
    /// - On success, responds with [`GetLfTagExpressionOutput`](crate::operation::get_lf_tag_expression::GetLfTagExpressionOutput) with field(s):
    ///   - [`name(Option<String>)`](crate::operation::get_lf_tag_expression::GetLfTagExpressionOutput::name): <p>The name for the LF-Tag expression.</p>
    ///   - [`description(Option<String>)`](crate::operation::get_lf_tag_expression::GetLfTagExpressionOutput::description): <p>The description with information about the LF-Tag expression.</p>
    ///   - [`catalog_id(Option<String>)`](crate::operation::get_lf_tag_expression::GetLfTagExpressionOutput::catalog_id): <p>The identifier for the Data Catalog. By default, the account ID in which the LF-Tag expression is saved.</p>
    ///   - [`expression(Option<Vec::<LfTag>>)`](crate::operation::get_lf_tag_expression::GetLfTagExpressionOutput::expression): <p>The body of the LF-Tag expression. It is composed of one or more LF-Tag key-value pairs.</p>
    /// - On failure, responds with [`SdkError<GetLFTagExpressionError>`](crate::operation::get_lf_tag_expression::GetLFTagExpressionError)
    pub fn get_lf_tag_expression(&self) -> crate::operation::get_lf_tag_expression::builders::GetLFTagExpressionFluentBuilder {
        crate::operation::get_lf_tag_expression::builders::GetLFTagExpressionFluentBuilder::new(self.handle.clone())
    }
}
