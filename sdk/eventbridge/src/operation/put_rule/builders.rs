// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::put_rule::_put_rule_output::PutRuleOutputBuilder;

pub use crate::operation::put_rule::_put_rule_input::PutRuleInputBuilder;

impl crate::operation::put_rule::builders::PutRuleInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::put_rule::PutRuleOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::put_rule::PutRuleError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.put_rule();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `PutRule`.
///
/// <p>Creates or updates the specified rule. Rules are enabled by default, or based on value of the state. You can disable a rule using <a href="https://docs.aws.amazon.com/eventbridge/latest/APIReference/API_DisableRule.html">DisableRule</a>.</p>
/// <p>A single rule watches for events from a single event bus. Events generated by Amazon Web Services services go to your account's default event bus. Events generated by SaaS partner services or applications go to the matching partner event bus. If you have custom applications or services, you can specify whether their events go to your default event bus or a custom event bus that you have created. For more information, see <a href="https://docs.aws.amazon.com/eventbridge/latest/APIReference/API_CreateEventBus.html">CreateEventBus</a>.</p>
/// <p>If you are updating an existing rule, the rule is replaced with what you specify in this <code>PutRule</code> command. If you omit arguments in <code>PutRule</code>, the old values for those arguments are not kept. Instead, they are replaced with null values.</p>
/// <p>When you create or update a rule, incoming events might not immediately start matching to new or updated rules. Allow a short period of time for changes to take effect.</p>
/// <p>A rule must contain at least an EventPattern or ScheduleExpression. Rules with EventPatterns are triggered when a matching event is observed. Rules with ScheduleExpressions self-trigger based on the given schedule. A rule can have both an EventPattern and a ScheduleExpression, in which case the rule triggers on matching events as well as on a schedule.</p>
/// <p>When you initially create a rule, you can optionally assign one or more tags to the rule. Tags can help you organize and categorize your resources. You can also use them to scope user permissions, by granting a user permission to access or change only rules with certain tag values. To use the <code>PutRule</code> operation and assign tags, you must have both the <code>events:PutRule</code> and <code>events:TagResource</code> permissions.</p>
/// <p>If you are updating an existing rule, any tags you specify in the <code>PutRule</code> operation are ignored. To update the tags of an existing rule, use <a href="https://docs.aws.amazon.com/eventbridge/latest/APIReference/API_TagResource.html">TagResource</a> and <a href="https://docs.aws.amazon.com/eventbridge/latest/APIReference/API_UntagResource.html">UntagResource</a>.</p>
/// <p>Most services in Amazon Web Services treat : or / as the same character in Amazon Resource Names (ARNs). However, EventBridge uses an exact match in event patterns and rules. Be sure to use the correct ARN characters when creating event patterns so that they match the ARN syntax in the event you want to match.</p>
/// <p>In EventBridge, it is possible to create rules that lead to infinite loops, where a rule is fired repeatedly. For example, a rule might detect that ACLs have changed on an S3 bucket, and trigger software to change them to the desired state. If the rule is not written carefully, the subsequent change to the ACLs fires the rule again, creating an infinite loop.</p>
/// <p>To prevent this, write the rules so that the triggered actions do not re-fire the same rule. For example, your rule could fire only if ACLs are found to be in a bad state, instead of after any change.</p>
/// <p>An infinite loop can quickly cause higher than expected charges. We recommend that you use budgeting, which alerts you when charges exceed your specified limit. For more information, see <a href="https://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/budgets-managing-costs.html">Managing Your Costs with Budgets</a>.</p>
/// <p>To create a rule that filters for management events from Amazon Web Services services, see <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-service-event-cloudtrail.html#eb-service-event-cloudtrail-management">Receiving read-only management events from Amazon Web Services services</a> in the <i>EventBridge User Guide</i>.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct PutRuleFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::put_rule::builders::PutRuleInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl crate::client::customize::internal::CustomizableSend<crate::operation::put_rule::PutRuleOutput, crate::operation::put_rule::PutRuleError>
    for PutRuleFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<crate::operation::put_rule::PutRuleOutput, crate::operation::put_rule::PutRuleError>,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl PutRuleFluentBuilder {
    /// Creates a new `PutRuleFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the PutRule as a reference.
    pub fn as_input(&self) -> &crate::operation::put_rule::builders::PutRuleInputBuilder {
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
        crate::operation::put_rule::PutRuleOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::put_rule::PutRuleError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::put_rule::PutRule::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::put_rule::PutRule::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<crate::operation::put_rule::PutRuleOutput, crate::operation::put_rule::PutRuleError, Self>
    {
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
    /// <p>The name of the rule that you are creating or updating.</p>
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.name(input.into());
        self
    }
    /// <p>The name of the rule that you are creating or updating.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_name(input);
        self
    }
    /// <p>The name of the rule that you are creating or updating.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_name()
    }
    /// <p>The scheduling expression. For example, "cron(0 20 * * ? *)" or "rate(5 minutes)".</p>
    pub fn schedule_expression(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.schedule_expression(input.into());
        self
    }
    /// <p>The scheduling expression. For example, "cron(0 20 * * ? *)" or "rate(5 minutes)".</p>
    pub fn set_schedule_expression(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_schedule_expression(input);
        self
    }
    /// <p>The scheduling expression. For example, "cron(0 20 * * ? *)" or "rate(5 minutes)".</p>
    pub fn get_schedule_expression(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_schedule_expression()
    }
    /// <p>The event pattern. For more information, see <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-event-patterns.html">Amazon EventBridge event patterns</a> in the <i> <i>Amazon EventBridge User Guide</i> </i>.</p>
    pub fn event_pattern(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.event_pattern(input.into());
        self
    }
    /// <p>The event pattern. For more information, see <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-event-patterns.html">Amazon EventBridge event patterns</a> in the <i> <i>Amazon EventBridge User Guide</i> </i>.</p>
    pub fn set_event_pattern(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_event_pattern(input);
        self
    }
    /// <p>The event pattern. For more information, see <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-event-patterns.html">Amazon EventBridge event patterns</a> in the <i> <i>Amazon EventBridge User Guide</i> </i>.</p>
    pub fn get_event_pattern(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_event_pattern()
    }
    /// <p>The state of the rule.</p>
    /// <p>Valid values include:</p>
    /// <ul>
    /// <li>
    /// <p><code>DISABLED</code>: The rule is disabled. EventBridge does not match any events against the rule.</p></li>
    /// <li>
    /// <p><code>ENABLED</code>: The rule is enabled. EventBridge matches events against the rule, <i>except</i> for Amazon Web Services management events delivered through CloudTrail.</p></li>
    /// <li>
    /// <p><code>ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS</code>: The rule is enabled for all events, including Amazon Web Services management events delivered through CloudTrail.</p>
    /// <p>Management events provide visibility into management operations that are performed on resources in your Amazon Web Services account. These are also known as control plane operations. For more information, see <a href="https://docs.aws.amazon.com/awscloudtrail/latest/userguide/logging-management-events-with-cloudtrail.html#logging-management-events">Logging management events</a> in the <i>CloudTrail User Guide</i>, and <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-service-event.html#eb-service-event-cloudtrail">Filtering management events from Amazon Web Services services</a> in the <i> <i>Amazon EventBridge User Guide</i> </i>.</p>
    /// <p>This value is only valid for rules on the <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-what-is-how-it-works-concepts.html#eb-bus-concepts-buses">default</a> event bus or <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-create-event-bus.html">custom event buses</a>. It does not apply to <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-saas.html">partner event buses</a>.</p></li>
    /// </ul>
    pub fn state(mut self, input: crate::types::RuleState) -> Self {
        self.inner = self.inner.state(input);
        self
    }
    /// <p>The state of the rule.</p>
    /// <p>Valid values include:</p>
    /// <ul>
    /// <li>
    /// <p><code>DISABLED</code>: The rule is disabled. EventBridge does not match any events against the rule.</p></li>
    /// <li>
    /// <p><code>ENABLED</code>: The rule is enabled. EventBridge matches events against the rule, <i>except</i> for Amazon Web Services management events delivered through CloudTrail.</p></li>
    /// <li>
    /// <p><code>ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS</code>: The rule is enabled for all events, including Amazon Web Services management events delivered through CloudTrail.</p>
    /// <p>Management events provide visibility into management operations that are performed on resources in your Amazon Web Services account. These are also known as control plane operations. For more information, see <a href="https://docs.aws.amazon.com/awscloudtrail/latest/userguide/logging-management-events-with-cloudtrail.html#logging-management-events">Logging management events</a> in the <i>CloudTrail User Guide</i>, and <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-service-event.html#eb-service-event-cloudtrail">Filtering management events from Amazon Web Services services</a> in the <i> <i>Amazon EventBridge User Guide</i> </i>.</p>
    /// <p>This value is only valid for rules on the <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-what-is-how-it-works-concepts.html#eb-bus-concepts-buses">default</a> event bus or <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-create-event-bus.html">custom event buses</a>. It does not apply to <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-saas.html">partner event buses</a>.</p></li>
    /// </ul>
    pub fn set_state(mut self, input: ::std::option::Option<crate::types::RuleState>) -> Self {
        self.inner = self.inner.set_state(input);
        self
    }
    /// <p>The state of the rule.</p>
    /// <p>Valid values include:</p>
    /// <ul>
    /// <li>
    /// <p><code>DISABLED</code>: The rule is disabled. EventBridge does not match any events against the rule.</p></li>
    /// <li>
    /// <p><code>ENABLED</code>: The rule is enabled. EventBridge matches events against the rule, <i>except</i> for Amazon Web Services management events delivered through CloudTrail.</p></li>
    /// <li>
    /// <p><code>ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS</code>: The rule is enabled for all events, including Amazon Web Services management events delivered through CloudTrail.</p>
    /// <p>Management events provide visibility into management operations that are performed on resources in your Amazon Web Services account. These are also known as control plane operations. For more information, see <a href="https://docs.aws.amazon.com/awscloudtrail/latest/userguide/logging-management-events-with-cloudtrail.html#logging-management-events">Logging management events</a> in the <i>CloudTrail User Guide</i>, and <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-service-event.html#eb-service-event-cloudtrail">Filtering management events from Amazon Web Services services</a> in the <i> <i>Amazon EventBridge User Guide</i> </i>.</p>
    /// <p>This value is only valid for rules on the <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-what-is-how-it-works-concepts.html#eb-bus-concepts-buses">default</a> event bus or <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-create-event-bus.html">custom event buses</a>. It does not apply to <a href="https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-saas.html">partner event buses</a>.</p></li>
    /// </ul>
    pub fn get_state(&self) -> &::std::option::Option<crate::types::RuleState> {
        self.inner.get_state()
    }
    /// <p>A description of the rule.</p>
    pub fn description(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.description(input.into());
        self
    }
    /// <p>A description of the rule.</p>
    pub fn set_description(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_description(input);
        self
    }
    /// <p>A description of the rule.</p>
    pub fn get_description(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_description()
    }
    /// <p>The Amazon Resource Name (ARN) of the IAM role associated with the rule.</p>
    /// <p>If you're setting an event bus in another account as the target and that account granted permission to your account through an organization instead of directly by the account ID, you must specify a <code>RoleArn</code> with proper permissions in the <code>Target</code> structure, instead of here in this parameter.</p>
    pub fn role_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.role_arn(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the IAM role associated with the rule.</p>
    /// <p>If you're setting an event bus in another account as the target and that account granted permission to your account through an organization instead of directly by the account ID, you must specify a <code>RoleArn</code> with proper permissions in the <code>Target</code> structure, instead of here in this parameter.</p>
    pub fn set_role_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_role_arn(input);
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the IAM role associated with the rule.</p>
    /// <p>If you're setting an event bus in another account as the target and that account granted permission to your account through an organization instead of directly by the account ID, you must specify a <code>RoleArn</code> with proper permissions in the <code>Target</code> structure, instead of here in this parameter.</p>
    pub fn get_role_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_role_arn()
    }
    ///
    /// Appends an item to `Tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>The list of key-value pairs to associate with the rule.</p>
    pub fn tags(mut self, input: crate::types::Tag) -> Self {
        self.inner = self.inner.tags(input);
        self
    }
    /// <p>The list of key-value pairs to associate with the rule.</p>
    pub fn set_tags(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Tag>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>The list of key-value pairs to associate with the rule.</p>
    pub fn get_tags(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Tag>> {
        self.inner.get_tags()
    }
    /// <p>The name or ARN of the event bus to associate with this rule. If you omit this, the default event bus is used.</p>
    pub fn event_bus_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.event_bus_name(input.into());
        self
    }
    /// <p>The name or ARN of the event bus to associate with this rule. If you omit this, the default event bus is used.</p>
    pub fn set_event_bus_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_event_bus_name(input);
        self
    }
    /// <p>The name or ARN of the event bus to associate with this rule. If you omit this, the default event bus is used.</p>
    pub fn get_event_bus_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_event_bus_name()
    }
}
