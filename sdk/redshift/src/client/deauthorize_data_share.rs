// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DeauthorizeDataShare`](crate::operation::deauthorize_data_share::builders::DeauthorizeDataShareFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`data_share_arn(impl Into<String>)`](crate::operation::deauthorize_data_share::builders::DeauthorizeDataShareFluentBuilder::data_share_arn) / [`set_data_share_arn(Option<String>)`](crate::operation::deauthorize_data_share::builders::DeauthorizeDataShareFluentBuilder::set_data_share_arn):<br>required: **true**<br><p>The namespace Amazon Resource Name (ARN) of the datashare to remove authorization from.</p><br>
    ///   - [`consumer_identifier(impl Into<String>)`](crate::operation::deauthorize_data_share::builders::DeauthorizeDataShareFluentBuilder::consumer_identifier) / [`set_consumer_identifier(Option<String>)`](crate::operation::deauthorize_data_share::builders::DeauthorizeDataShareFluentBuilder::set_consumer_identifier):<br>required: **true**<br><p>The identifier of the data consumer that is to have authorization removed from the datashare. This identifier is an Amazon Web Services account ID or a keyword, such as ADX.</p><br>
    /// - On success, responds with [`DeauthorizeDataShareOutput`](crate::operation::deauthorize_data_share::DeauthorizeDataShareOutput) with field(s):
    ///   - [`data_share_arn(Option<String>)`](crate::operation::deauthorize_data_share::DeauthorizeDataShareOutput::data_share_arn): <p>The Amazon Resource Name (ARN) of the datashare that the consumer is to use.</p>
    ///   - [`producer_arn(Option<String>)`](crate::operation::deauthorize_data_share::DeauthorizeDataShareOutput::producer_arn): <p>The Amazon Resource Name (ARN) of the producer namespace.</p>
    ///   - [`allow_publicly_accessible_consumers(Option<bool>)`](crate::operation::deauthorize_data_share::DeauthorizeDataShareOutput::allow_publicly_accessible_consumers): <p>A value that specifies whether the datashare can be shared to a publicly accessible cluster.</p>
    ///   - [`data_share_associations(Option<Vec::<DataShareAssociation>>)`](crate::operation::deauthorize_data_share::DeauthorizeDataShareOutput::data_share_associations): <p>A value that specifies when the datashare has an association between producer and data consumers.</p>
    ///   - [`managed_by(Option<String>)`](crate::operation::deauthorize_data_share::DeauthorizeDataShareOutput::managed_by): <p>The identifier of a datashare to show its managing entity.</p>
    ///   - [`data_share_type(Option<DataShareType>)`](crate::operation::deauthorize_data_share::DeauthorizeDataShareOutput::data_share_type): <p>The type of the datashare created by RegisterNamespace.</p>
    /// - On failure, responds with [`SdkError<DeauthorizeDataShareError>`](crate::operation::deauthorize_data_share::DeauthorizeDataShareError)
    pub fn deauthorize_data_share(&self) -> crate::operation::deauthorize_data_share::builders::DeauthorizeDataShareFluentBuilder {
        crate::operation::deauthorize_data_share::builders::DeauthorizeDataShareFluentBuilder::new(self.handle.clone())
    }
}
