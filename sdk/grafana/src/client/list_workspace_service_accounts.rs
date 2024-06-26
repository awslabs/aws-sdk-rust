// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListWorkspaceServiceAccounts`](crate::operation::list_workspace_service_accounts::builders::ListWorkspaceServiceAccountsFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_workspace_service_accounts::builders::ListWorkspaceServiceAccountsFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`max_results(i32)`](crate::operation::list_workspace_service_accounts::builders::ListWorkspaceServiceAccountsFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_workspace_service_accounts::builders::ListWorkspaceServiceAccountsFluentBuilder::set_max_results):<br>required: **false**<br><p>The maximum number of service accounts to include in the results.</p><br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_workspace_service_accounts::builders::ListWorkspaceServiceAccountsFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_workspace_service_accounts::builders::ListWorkspaceServiceAccountsFluentBuilder::set_next_token):<br>required: **false**<br><p>The token for the next set of service accounts to return. (You receive this token from a previous <code>ListWorkspaceServiceAccounts</code> operation.)</p><br>
    ///   - [`workspace_id(impl Into<String>)`](crate::operation::list_workspace_service_accounts::builders::ListWorkspaceServiceAccountsFluentBuilder::workspace_id) / [`set_workspace_id(Option<String>)`](crate::operation::list_workspace_service_accounts::builders::ListWorkspaceServiceAccountsFluentBuilder::set_workspace_id):<br>required: **true**<br><p>The workspace for which to list service accounts.</p><br>
    /// - On success, responds with [`ListWorkspaceServiceAccountsOutput`](crate::operation::list_workspace_service_accounts::ListWorkspaceServiceAccountsOutput) with field(s):
    ///   - [`next_token(Option<String>)`](crate::operation::list_workspace_service_accounts::ListWorkspaceServiceAccountsOutput::next_token): <p>The token to use when requesting the next set of service accounts.</p>
    ///   - [`service_accounts(Vec::<ServiceAccountSummary>)`](crate::operation::list_workspace_service_accounts::ListWorkspaceServiceAccountsOutput::service_accounts): <p>An array of structures containing information about the service accounts.</p>
    ///   - [`workspace_id(String)`](crate::operation::list_workspace_service_accounts::ListWorkspaceServiceAccountsOutput::workspace_id): <p>The workspace to which the service accounts are associated.</p>
    /// - On failure, responds with [`SdkError<ListWorkspaceServiceAccountsError>`](crate::operation::list_workspace_service_accounts::ListWorkspaceServiceAccountsError)
    pub fn list_workspace_service_accounts(
        &self,
    ) -> crate::operation::list_workspace_service_accounts::builders::ListWorkspaceServiceAccountsFluentBuilder {
        crate::operation::list_workspace_service_accounts::builders::ListWorkspaceServiceAccountsFluentBuilder::new(self.handle.clone())
    }
}
