// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListInvalidationsForDistributionTenant`](crate::operation::list_invalidations_for_distribution_tenant::builders::ListInvalidationsForDistributionTenantFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_invalidations_for_distribution_tenant::builders::ListInvalidationsForDistributionTenantFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`id(impl Into<String>)`](crate::operation::list_invalidations_for_distribution_tenant::builders::ListInvalidationsForDistributionTenantFluentBuilder::id) / [`set_id(Option<String>)`](crate::operation::list_invalidations_for_distribution_tenant::builders::ListInvalidationsForDistributionTenantFluentBuilder::set_id):<br>required: **true**<br><p>The ID of the distribution tenant.</p><br>
    ///   - [`marker(impl Into<String>)`](crate::operation::list_invalidations_for_distribution_tenant::builders::ListInvalidationsForDistributionTenantFluentBuilder::marker) / [`set_marker(Option<String>)`](crate::operation::list_invalidations_for_distribution_tenant::builders::ListInvalidationsForDistributionTenantFluentBuilder::set_marker):<br>required: **false**<br><p>Use this parameter when paginating results to indicate where to begin in your list of invalidation batches. Because the results are returned in decreasing order from most recent to oldest, the most recent results are on the first page, the second page will contain earlier results, and so on. To get the next page of results, set <code>Marker</code> to the value of the <code>NextMarker</code> from the current page's response. This value is the same as the ID of the last invalidation batch on that page.</p><br>
    ///   - [`max_items(i32)`](crate::operation::list_invalidations_for_distribution_tenant::builders::ListInvalidationsForDistributionTenantFluentBuilder::max_items) / [`set_max_items(Option<i32>)`](crate::operation::list_invalidations_for_distribution_tenant::builders::ListInvalidationsForDistributionTenantFluentBuilder::set_max_items):<br>required: **false**<br><p>The maximum number of invalidations to return for the distribution tenant.</p><br>
    /// - On success, responds with [`ListInvalidationsForDistributionTenantOutput`](crate::operation::list_invalidations_for_distribution_tenant::ListInvalidationsForDistributionTenantOutput) with field(s):
    ///   - [`invalidation_list(Option<InvalidationList>)`](crate::operation::list_invalidations_for_distribution_tenant::ListInvalidationsForDistributionTenantOutput::invalidation_list): <p>The <code>InvalidationList</code> complex type describes the list of invalidation objects. For more information about invalidation, see <a href="https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/Invalidation.html">Invalidating Objects (Web Distributions Only)</a> in the <i>Amazon CloudFront Developer Guide</i>.</p>
    /// - On failure, responds with [`SdkError<ListInvalidationsForDistributionTenantError>`](crate::operation::list_invalidations_for_distribution_tenant::ListInvalidationsForDistributionTenantError)
    pub fn list_invalidations_for_distribution_tenant(
        &self,
    ) -> crate::operation::list_invalidations_for_distribution_tenant::builders::ListInvalidationsForDistributionTenantFluentBuilder {
        crate::operation::list_invalidations_for_distribution_tenant::builders::ListInvalidationsForDistributionTenantFluentBuilder::new(
            self.handle.clone(),
        )
    }
}
