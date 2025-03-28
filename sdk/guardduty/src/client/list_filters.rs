// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListFilters`](crate::operation::list_filters::builders::ListFiltersFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_filters::builders::ListFiltersFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`detector_id(impl Into<String>)`](crate::operation::list_filters::builders::ListFiltersFluentBuilder::detector_id) / [`set_detector_id(Option<String>)`](crate::operation::list_filters::builders::ListFiltersFluentBuilder::set_detector_id):<br>required: **true**<br><p>The unique ID of the detector that is associated with the filter.</p> <p>To find the <code>detectorId</code> in the current Region, see the Settings page in the GuardDuty console, or run the <a href="https://docs.aws.amazon.com/guardduty/latest/APIReference/API_ListDetectors.html">ListDetectors</a> API.</p><br>
    ///   - [`max_results(i32)`](crate::operation::list_filters::builders::ListFiltersFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_filters::builders::ListFiltersFluentBuilder::set_max_results):<br>required: **false**<br><p>You can use this parameter to indicate the maximum number of items that you want in the response. The default value is 50. The maximum value is 50.</p><br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_filters::builders::ListFiltersFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_filters::builders::ListFiltersFluentBuilder::set_next_token):<br>required: **false**<br><p>You can use this parameter when paginating results. Set the value of this parameter to null on your first call to the list action. For subsequent calls to the action, fill nextToken in the request with the value of NextToken from the previous response to continue listing data.</p><br>
    /// - On success, responds with [`ListFiltersOutput`](crate::operation::list_filters::ListFiltersOutput) with field(s):
    ///   - [`filter_names(Option<Vec::<String>>)`](crate::operation::list_filters::ListFiltersOutput::filter_names): <p>A list of filter names.</p>
    ///   - [`next_token(Option<String>)`](crate::operation::list_filters::ListFiltersOutput::next_token): <p>The pagination parameter to be used on the next list operation to retrieve more items.</p>
    /// - On failure, responds with [`SdkError<ListFiltersError>`](crate::operation::list_filters::ListFiltersError)
    pub fn list_filters(&self) -> crate::operation::list_filters::builders::ListFiltersFluentBuilder {
        crate::operation::list_filters::builders::ListFiltersFluentBuilder::new(self.handle.clone())
    }
}
