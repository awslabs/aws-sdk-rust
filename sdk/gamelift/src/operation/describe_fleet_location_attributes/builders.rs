// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::describe_fleet_location_attributes::_describe_fleet_location_attributes_output::DescribeFleetLocationAttributesOutputBuilder;

pub use crate::operation::describe_fleet_location_attributes::_describe_fleet_location_attributes_input::DescribeFleetLocationAttributesInputBuilder;

impl crate::operation::describe_fleet_location_attributes::builders::DescribeFleetLocationAttributesInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributesOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributesError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.describe_fleet_location_attributes();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `DescribeFleetLocationAttributes`.
///
/// <p>Retrieves information on a fleet's remote locations, including life-cycle status and any suspended fleet activity.</p>
/// <p>This operation can be used in the following ways:</p>
/// <ul>
/// <li>
/// <p>To get data for specific locations, provide a fleet identifier and a list of locations. Location data is returned in the order that it is requested.</p></li>
/// <li>
/// <p>To get data for all locations, provide a fleet identifier only. Location data is returned in no particular order.</p></li>
/// </ul>
/// <p>When requesting attributes for multiple locations, use the pagination parameters to retrieve results as a set of sequential pages.</p>
/// <p>If successful, a <code>LocationAttributes</code> object is returned for each requested location. If the fleet does not have a requested location, no information is returned. This operation does not return the home Region. To get information on a fleet's home Region, call <code>DescribeFleetAttributes</code>.</p>
/// <p><b>Learn more</b></p>
/// <p><a href="https://docs.aws.amazon.com/gamelift/latest/developerguide/fleets-intro.html">Setting up Amazon GameLift Servers fleets</a></p>
/// <p><a href="https://docs.aws.amazon.com/gamelift/latest/developerguide/gamelift-regions.html"> Amazon GameLift Servers service locations</a> for managed hosting</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct DescribeFleetLocationAttributesFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::describe_fleet_location_attributes::builders::DescribeFleetLocationAttributesInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributesOutput,
        crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributesError,
    > for DescribeFleetLocationAttributesFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributesOutput,
            crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributesError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl DescribeFleetLocationAttributesFluentBuilder {
    /// Creates a new `DescribeFleetLocationAttributesFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the DescribeFleetLocationAttributes as a reference.
    pub fn as_input(&self) -> &crate::operation::describe_fleet_location_attributes::builders::DescribeFleetLocationAttributesInputBuilder {
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
        crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributesOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributesError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributes::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributes::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributesOutput,
        crate::operation::describe_fleet_location_attributes::DescribeFleetLocationAttributesError,
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
    /// Create a paginator for this request
    ///
    /// Paginators are used by calling [`send().await`](crate::operation::describe_fleet_location_attributes::paginator::DescribeFleetLocationAttributesPaginator::send) which returns a [`PaginationStream`](aws_smithy_async::future::pagination_stream::PaginationStream).
    pub fn into_paginator(self) -> crate::operation::describe_fleet_location_attributes::paginator::DescribeFleetLocationAttributesPaginator {
        crate::operation::describe_fleet_location_attributes::paginator::DescribeFleetLocationAttributesPaginator::new(self.handle, self.inner)
    }
    /// <p>A unique identifier for the fleet to retrieve remote locations for. You can use either the fleet ID or ARN value.</p>
    pub fn fleet_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.fleet_id(input.into());
        self
    }
    /// <p>A unique identifier for the fleet to retrieve remote locations for. You can use either the fleet ID or ARN value.</p>
    pub fn set_fleet_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_fleet_id(input);
        self
    }
    /// <p>A unique identifier for the fleet to retrieve remote locations for. You can use either the fleet ID or ARN value.</p>
    pub fn get_fleet_id(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_fleet_id()
    }
    ///
    /// Appends an item to `Locations`.
    ///
    /// To override the contents of this collection use [`set_locations`](Self::set_locations).
    ///
    /// <p>A list of fleet locations to retrieve information for. Specify locations in the form of an Amazon Web Services Region code, such as <code>us-west-2</code>.</p>
    pub fn locations(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.locations(input.into());
        self
    }
    /// <p>A list of fleet locations to retrieve information for. Specify locations in the form of an Amazon Web Services Region code, such as <code>us-west-2</code>.</p>
    pub fn set_locations(mut self, input: ::std::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        self.inner = self.inner.set_locations(input);
        self
    }
    /// <p>A list of fleet locations to retrieve information for. Specify locations in the form of an Amazon Web Services Region code, such as <code>us-west-2</code>.</p>
    pub fn get_locations(&self) -> &::std::option::Option<::std::vec::Vec<::std::string::String>> {
        self.inner.get_locations()
    }
    /// <p>The maximum number of results to return. Use this parameter with <code>NextToken</code> to get results as a set of sequential pages. This limit is not currently enforced.</p>
    pub fn limit(mut self, input: i32) -> Self {
        self.inner = self.inner.limit(input);
        self
    }
    /// <p>The maximum number of results to return. Use this parameter with <code>NextToken</code> to get results as a set of sequential pages. This limit is not currently enforced.</p>
    pub fn set_limit(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_limit(input);
        self
    }
    /// <p>The maximum number of results to return. Use this parameter with <code>NextToken</code> to get results as a set of sequential pages. This limit is not currently enforced.</p>
    pub fn get_limit(&self) -> &::std::option::Option<i32> {
        self.inner.get_limit()
    }
    /// <p>A token that indicates the start of the next sequential page of results. Use the token that is returned with a previous call to this operation. To start at the beginning of the result set, do not specify a value.</p>
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.next_token(input.into());
        self
    }
    /// <p>A token that indicates the start of the next sequential page of results. Use the token that is returned with a previous call to this operation. To start at the beginning of the result set, do not specify a value.</p>
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_next_token(input);
        self
    }
    /// <p>A token that indicates the start of the next sequential page of results. Use the token that is returned with a previous call to this operation. To start at the beginning of the result set, do not specify a value.</p>
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_next_token()
    }
}
