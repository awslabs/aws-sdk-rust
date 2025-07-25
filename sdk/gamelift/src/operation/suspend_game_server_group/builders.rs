// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::suspend_game_server_group::_suspend_game_server_group_output::SuspendGameServerGroupOutputBuilder;

pub use crate::operation::suspend_game_server_group::_suspend_game_server_group_input::SuspendGameServerGroupInputBuilder;

impl crate::operation::suspend_game_server_group::builders::SuspendGameServerGroupInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::suspend_game_server_group::SuspendGameServerGroupOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::suspend_game_server_group::SuspendGameServerGroupError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.suspend_game_server_group();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `SuspendGameServerGroup`.
///
/// <p><b>This operation is used with the Amazon GameLift Servers FleetIQ solution and game server groups.</b></p>
/// <p>Temporarily stops activity on a game server group without terminating instances or the game server group. You can restart activity by calling <a href="gamelift/latest/apireference/API_ResumeGameServerGroup.html">ResumeGameServerGroup</a>. You can suspend the following activity:</p>
/// <ul>
/// <li>
/// <p><b>Instance type replacement</b> - This activity evaluates the current game hosting viability of all Spot instance types that are defined for the game server group. It updates the Auto Scaling group to remove nonviable Spot Instance types, which have a higher chance of game server interruptions. It then balances capacity across the remaining viable Spot Instance types. When this activity is suspended, the Auto Scaling group continues with its current balance, regardless of viability. Instance protection, utilization metrics, and capacity scaling activities continue to be active.</p></li>
/// </ul>
/// <p>To suspend activity, specify a game server group ARN and the type of activity to be suspended. If successful, a <code>GameServerGroup</code> object is returned showing that the activity is listed in <code>SuspendedActions</code>.</p>
/// <p><b>Learn more</b></p>
/// <p><a href="https://docs.aws.amazon.com/gamelift/latest/fleetiqguide/gsg-intro.html">Amazon GameLift Servers FleetIQ Guide</a></p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct SuspendGameServerGroupFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::suspend_game_server_group::builders::SuspendGameServerGroupInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::suspend_game_server_group::SuspendGameServerGroupOutput,
        crate::operation::suspend_game_server_group::SuspendGameServerGroupError,
    > for SuspendGameServerGroupFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::suspend_game_server_group::SuspendGameServerGroupOutput,
            crate::operation::suspend_game_server_group::SuspendGameServerGroupError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl SuspendGameServerGroupFluentBuilder {
    /// Creates a new `SuspendGameServerGroupFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the SuspendGameServerGroup as a reference.
    pub fn as_input(&self) -> &crate::operation::suspend_game_server_group::builders::SuspendGameServerGroupInputBuilder {
        &self.inner
    }
    /// Sends the request and returns the response.
    ///
    /// If an error occurs, an `SdkError` will be returned with additional details that
    /// can be matched against.
    ///
    /// By default, any retryable failures will be retried twice. Retry behavior
    /// is configurable with the [RetryConfig](aws_smithy_types::retry::RetryConfig), which can be
    /// set when configuring the client.
    pub async fn send(
        self,
    ) -> ::std::result::Result<
        crate::operation::suspend_game_server_group::SuspendGameServerGroupOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::suspend_game_server_group::SuspendGameServerGroupError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::suspend_game_server_group::SuspendGameServerGroup::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::suspend_game_server_group::SuspendGameServerGroup::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::suspend_game_server_group::SuspendGameServerGroupOutput,
        crate::operation::suspend_game_server_group::SuspendGameServerGroupError,
        Self,
    > {
        crate::client::customize::CustomizableOperation::new(self)
    }
    pub(crate) fn config_override(mut self, config_override: impl ::std::convert::Into<crate::config::Builder>) -> Self {
        self.set_config_override(::std::option::Option::Some(config_override.into()));
        self
    }

    pub(crate) fn set_config_override(&mut self, config_override: ::std::option::Option<crate::config::Builder>) -> &mut Self {
        self.config_override = config_override;
        self
    }
    /// <p>A unique identifier for the game server group. Use either the name or ARN value.</p>
    pub fn game_server_group_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.game_server_group_name(input.into());
        self
    }
    /// <p>A unique identifier for the game server group. Use either the name or ARN value.</p>
    pub fn set_game_server_group_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_game_server_group_name(input);
        self
    }
    /// <p>A unique identifier for the game server group. Use either the name or ARN value.</p>
    pub fn get_game_server_group_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_game_server_group_name()
    }
    ///
    /// Appends an item to `SuspendActions`.
    ///
    /// To override the contents of this collection use [`set_suspend_actions`](Self::set_suspend_actions).
    ///
    /// <p>The activity to suspend for this game server group.</p>
    pub fn suspend_actions(mut self, input: crate::types::GameServerGroupAction) -> Self {
        self.inner = self.inner.suspend_actions(input);
        self
    }
    /// <p>The activity to suspend for this game server group.</p>
    pub fn set_suspend_actions(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::GameServerGroupAction>>) -> Self {
        self.inner = self.inner.set_suspend_actions(input);
        self
    }
    /// <p>The activity to suspend for this game server group.</p>
    pub fn get_suspend_actions(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::GameServerGroupAction>> {
        self.inner.get_suspend_actions()
    }
}
