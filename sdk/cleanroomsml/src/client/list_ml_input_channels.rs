// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListMLInputChannels`](crate::operation::list_ml_input_channels::builders::ListMLInputChannelsFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_ml_input_channels::builders::ListMLInputChannelsFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_ml_input_channels::builders::ListMLInputChannelsFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_ml_input_channels::builders::ListMLInputChannelsFluentBuilder::set_next_token):<br>required: **false**<br><p>The token value retrieved from a previous call to access the next page of results.</p><br>
    ///   - [`max_results(i32)`](crate::operation::list_ml_input_channels::builders::ListMLInputChannelsFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_ml_input_channels::builders::ListMLInputChannelsFluentBuilder::set_max_results):<br>required: **false**<br><p>The maximum number of ML input channels to return.</p><br>
    ///   - [`membership_identifier(impl Into<String>)`](crate::operation::list_ml_input_channels::builders::ListMLInputChannelsFluentBuilder::membership_identifier) / [`set_membership_identifier(Option<String>)`](crate::operation::list_ml_input_channels::builders::ListMLInputChannelsFluentBuilder::set_membership_identifier):<br>required: **true**<br><p>The membership ID of the membership that contains the ML input channels that you want to list.</p><br>
    /// - On success, responds with [`ListMlInputChannelsOutput`](crate::operation::list_ml_input_channels::ListMlInputChannelsOutput) with field(s):
    ///   - [`next_token(Option<String>)`](crate::operation::list_ml_input_channels::ListMlInputChannelsOutput::next_token): <p>The token value used to access the next page of results.</p>
    ///   - [`ml_input_channels_list(Vec::<MlInputChannelSummary>)`](crate::operation::list_ml_input_channels::ListMlInputChannelsOutput::ml_input_channels_list): <p>The list of ML input channels that you wanted.</p>
    /// - On failure, responds with [`SdkError<ListMLInputChannelsError>`](crate::operation::list_ml_input_channels::ListMLInputChannelsError)
    pub fn list_ml_input_channels(&self) -> crate::operation::list_ml_input_channels::builders::ListMLInputChannelsFluentBuilder {
        crate::operation::list_ml_input_channels::builders::ListMLInputChannelsFluentBuilder::new(self.handle.clone())
    }
}
