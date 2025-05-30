// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListDataIntegrationFlowExecutions`](crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`instance_id(impl Into<String>)`](crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder::instance_id) / [`set_instance_id(Option<String>)`](crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder::set_instance_id):<br>required: **true**<br><p>The AWS Supply Chain instance identifier.</p><br>
    ///   - [`flow_name(impl Into<String>)`](crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder::flow_name) / [`set_flow_name(Option<String>)`](crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder::set_flow_name):<br>required: **true**<br><p>The flow name.</p><br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder::set_next_token):<br>required: **false**<br><p>The pagination token to fetch next page of flow executions.</p><br>
    ///   - [`max_results(i32)`](crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder::set_max_results):<br>required: **false**<br><p>The number to specify the max number of flow executions to fetch in this paginated request.</p><br>
    /// - On success, responds with [`ListDataIntegrationFlowExecutionsOutput`](crate::operation::list_data_integration_flow_executions::ListDataIntegrationFlowExecutionsOutput) with field(s):
    ///   - [`flow_executions(Vec::<DataIntegrationFlowExecution>)`](crate::operation::list_data_integration_flow_executions::ListDataIntegrationFlowExecutionsOutput::flow_executions): <p>The list of flow executions.</p>
    ///   - [`next_token(Option<String>)`](crate::operation::list_data_integration_flow_executions::ListDataIntegrationFlowExecutionsOutput::next_token): <p>The pagination token to fetch next page of flow executions.</p>
    /// - On failure, responds with [`SdkError<ListDataIntegrationFlowExecutionsError>`](crate::operation::list_data_integration_flow_executions::ListDataIntegrationFlowExecutionsError)
    pub fn list_data_integration_flow_executions(
        &self,
    ) -> crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder {
        crate::operation::list_data_integration_flow_executions::builders::ListDataIntegrationFlowExecutionsFluentBuilder::new(self.handle.clone())
    }
}
