// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`UpdateDataIntegrationFlow`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`instance_id(impl Into<String>)`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::instance_id) / [`set_instance_id(Option<String>)`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::set_instance_id):<br>required: **true**<br><p>The Amazon Web Services Supply Chain instance identifier.</p><br>
    ///   - [`name(impl Into<String>)`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::name) / [`set_name(Option<String>)`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::set_name):<br>required: **true**<br><p>The name of the DataIntegrationFlow to be updated.</p><br>
    ///   - [`sources(DataIntegrationFlowSource)`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::sources) / [`set_sources(Option<Vec::<DataIntegrationFlowSource>>)`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::set_sources):<br>required: **false**<br><p>The new source configurations for the DataIntegrationFlow.</p><br>
    ///   - [`transformation(DataIntegrationFlowTransformation)`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::transformation) / [`set_transformation(Option<DataIntegrationFlowTransformation>)`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::set_transformation):<br>required: **false**<br><p>The new transformation configurations for the DataIntegrationFlow.</p><br>
    ///   - [`target(DataIntegrationFlowTarget)`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::target) / [`set_target(Option<DataIntegrationFlowTarget>)`](crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::set_target):<br>required: **false**<br><p>The new target configurations for the DataIntegrationFlow.</p><br>
    /// - On success, responds with [`UpdateDataIntegrationFlowOutput`](crate::operation::update_data_integration_flow::UpdateDataIntegrationFlowOutput) with field(s):
    ///   - [`flow(Option<DataIntegrationFlow>)`](crate::operation::update_data_integration_flow::UpdateDataIntegrationFlowOutput::flow): <p>The details of the updated DataIntegrationFlow.</p>
    /// - On failure, responds with [`SdkError<UpdateDataIntegrationFlowError>`](crate::operation::update_data_integration_flow::UpdateDataIntegrationFlowError)
    pub fn update_data_integration_flow(&self) -> crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder {
        crate::operation::update_data_integration_flow::builders::UpdateDataIntegrationFlowFluentBuilder::new(self.handle.clone())
    }
}
