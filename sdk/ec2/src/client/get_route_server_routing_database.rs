// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetRouteServerRoutingDatabase`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`route_server_id(impl Into<String>)`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::route_server_id) / [`set_route_server_id(Option<String>)`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::set_route_server_id):<br>required: **true**<br><p>The ID of the route server for which to get the routing database.</p><br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::set_next_token):<br>required: **false**<br><p>The token for the next page of results.</p><br>
    ///   - [`max_results(i32)`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::set_max_results):<br>required: **false**<br><p>The maximum number of routing database entries to return in a single response.</p><br>
    ///   - [`dry_run(bool)`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::dry_run) / [`set_dry_run(Option<bool>)`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::set_dry_run):<br>required: **false**<br><p>A check for whether you have the required permissions for the action without actually making the request and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p><br>
    ///   - [`filters(Filter)`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::filters) / [`set_filters(Option<Vec::<Filter>>)`](crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::set_filters):<br>required: **false**<br><p>Filters to apply to the routing database query.</p><br>
    /// - On success, responds with [`GetRouteServerRoutingDatabaseOutput`](crate::operation::get_route_server_routing_database::GetRouteServerRoutingDatabaseOutput) with field(s):
    ///   - [`are_routes_persisted(Option<bool>)`](crate::operation::get_route_server_routing_database::GetRouteServerRoutingDatabaseOutput::are_routes_persisted): <p>Indicates whether routes are being persisted in the routing database.</p>
    ///   - [`routes(Option<Vec::<RouteServerRoute>>)`](crate::operation::get_route_server_routing_database::GetRouteServerRoutingDatabaseOutput::routes): <p>The collection of routes in the route server's routing database.</p>
    ///   - [`next_token(Option<String>)`](crate::operation::get_route_server_routing_database::GetRouteServerRoutingDatabaseOutput::next_token): <p>The token to use to retrieve the next page of results. This value is <code>null</code> when there are no more results to return.</p>
    /// - On failure, responds with [`SdkError<GetRouteServerRoutingDatabaseError>`](crate::operation::get_route_server_routing_database::GetRouteServerRoutingDatabaseError)
    pub fn get_route_server_routing_database(
        &self,
    ) -> crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder {
        crate::operation::get_route_server_routing_database::builders::GetRouteServerRoutingDatabaseFluentBuilder::new(self.handle.clone())
    }
}
