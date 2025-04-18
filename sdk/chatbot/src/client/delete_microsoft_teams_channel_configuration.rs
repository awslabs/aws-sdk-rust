// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DeleteMicrosoftTeamsChannelConfiguration`](crate::operation::delete_microsoft_teams_channel_configuration::builders::DeleteMicrosoftTeamsChannelConfigurationFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`chat_configuration_arn(impl Into<String>)`](crate::operation::delete_microsoft_teams_channel_configuration::builders::DeleteMicrosoftTeamsChannelConfigurationFluentBuilder::chat_configuration_arn) / [`set_chat_configuration_arn(Option<String>)`](crate::operation::delete_microsoft_teams_channel_configuration::builders::DeleteMicrosoftTeamsChannelConfigurationFluentBuilder::set_chat_configuration_arn):<br>required: **true**<br><p>The Amazon Resource Name (ARN) of the MicrosoftTeamsChannelConfiguration associated with the user identity to delete.</p><br>
    /// - On success, responds with [`DeleteMicrosoftTeamsChannelConfigurationOutput`](crate::operation::delete_microsoft_teams_channel_configuration::DeleteMicrosoftTeamsChannelConfigurationOutput)
    /// - On failure, responds with [`SdkError<DeleteMicrosoftTeamsChannelConfigurationError>`](crate::operation::delete_microsoft_teams_channel_configuration::DeleteMicrosoftTeamsChannelConfigurationError)
    pub fn delete_microsoft_teams_channel_configuration(
        &self,
    ) -> crate::operation::delete_microsoft_teams_channel_configuration::builders::DeleteMicrosoftTeamsChannelConfigurationFluentBuilder {
        crate::operation::delete_microsoft_teams_channel_configuration::builders::DeleteMicrosoftTeamsChannelConfigurationFluentBuilder::new(
            self.handle.clone(),
        )
    }
}
