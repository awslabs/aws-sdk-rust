// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`DisassociateRouteServer`](crate::operation::disassociate_route_server::builders::DisassociateRouteServerFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`route_server_id(impl Into<String>)`](crate::operation::disassociate_route_server::builders::DisassociateRouteServerFluentBuilder::route_server_id) / [`set_route_server_id(Option<String>)`](crate::operation::disassociate_route_server::builders::DisassociateRouteServerFluentBuilder::set_route_server_id):<br>required: **true**<br><p>The ID of the route server to disassociate.</p><br>
    ///   - [`vpc_id(impl Into<String>)`](crate::operation::disassociate_route_server::builders::DisassociateRouteServerFluentBuilder::vpc_id) / [`set_vpc_id(Option<String>)`](crate::operation::disassociate_route_server::builders::DisassociateRouteServerFluentBuilder::set_vpc_id):<br>required: **true**<br><p>The ID of the VPC to disassociate from the route server.</p><br>
    ///   - [`dry_run(bool)`](crate::operation::disassociate_route_server::builders::DisassociateRouteServerFluentBuilder::dry_run) / [`set_dry_run(Option<bool>)`](crate::operation::disassociate_route_server::builders::DisassociateRouteServerFluentBuilder::set_dry_run):<br>required: **false**<br><p>A check for whether you have the required permissions for the action without actually making the request and provides an error response. If you have the required permissions, the error response is <code>DryRunOperation</code>. Otherwise, it is <code>UnauthorizedOperation</code>.</p><br>
    /// - On success, responds with [`DisassociateRouteServerOutput`](crate::operation::disassociate_route_server::DisassociateRouteServerOutput) with field(s):
    ///   - [`route_server_association(Option<RouteServerAssociation>)`](crate::operation::disassociate_route_server::DisassociateRouteServerOutput::route_server_association): <p>Information about the disassociated route server.</p>
    /// - On failure, responds with [`SdkError<DisassociateRouteServerError>`](crate::operation::disassociate_route_server::DisassociateRouteServerError)
    pub fn disassociate_route_server(&self) -> crate::operation::disassociate_route_server::builders::DisassociateRouteServerFluentBuilder {
        crate::operation::disassociate_route_server::builders::DisassociateRouteServerFluentBuilder::new(self.handle.clone())
    }
}
