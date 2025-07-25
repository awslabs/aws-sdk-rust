// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`ListActors`](crate::operation::list_actors::builders::ListActorsFluentBuilder) operation.
    /// This operation supports pagination; See [`into_paginator()`](crate::operation::list_actors::builders::ListActorsFluentBuilder::into_paginator).
    ///
    /// - The fluent builder is configurable:
    ///   - [`memory_id(impl Into<String>)`](crate::operation::list_actors::builders::ListActorsFluentBuilder::memory_id) / [`set_memory_id(Option<String>)`](crate::operation::list_actors::builders::ListActorsFluentBuilder::set_memory_id):<br>required: **true**<br><p>The identifier of the memory store for which to list actors.</p><br>
    ///   - [`max_results(i32)`](crate::operation::list_actors::builders::ListActorsFluentBuilder::max_results) / [`set_max_results(Option<i32>)`](crate::operation::list_actors::builders::ListActorsFluentBuilder::set_max_results):<br>required: **false**<br><p>The maximum number of results to return in a single call. Minimum value of 1, maximum value of 100. Default is 20.</p><br>
    ///   - [`next_token(impl Into<String>)`](crate::operation::list_actors::builders::ListActorsFluentBuilder::next_token) / [`set_next_token(Option<String>)`](crate::operation::list_actors::builders::ListActorsFluentBuilder::set_next_token):<br>required: **false**<br><p>The token for the next set of results. Use the value returned in the previous response in the next request to retrieve the next set of results.</p><br>
    /// - On success, responds with [`ListActorsOutput`](crate::operation::list_actors::ListActorsOutput) with field(s):
    ///   - [`actor_summaries(Vec::<ActorSummary>)`](crate::operation::list_actors::ListActorsOutput::actor_summaries): <p>The list of actor summaries.</p>
    ///   - [`next_token(Option<String>)`](crate::operation::list_actors::ListActorsOutput::next_token): <p>The token to use in a subsequent request to get the next set of results. This value is null when there are no more results to return.</p>
    /// - On failure, responds with [`SdkError<ListActorsError>`](crate::operation::list_actors::ListActorsError)
    pub fn list_actors(&self) -> crate::operation::list_actors::builders::ListActorsFluentBuilder {
        crate::operation::list_actors::builders::ListActorsFluentBuilder::new(self.handle.clone())
    }
}
