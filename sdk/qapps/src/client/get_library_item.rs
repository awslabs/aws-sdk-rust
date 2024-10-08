// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetLibraryItem`](crate::operation::get_library_item::builders::GetLibraryItemFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`instance_id(impl Into<String>)`](crate::operation::get_library_item::builders::GetLibraryItemFluentBuilder::instance_id) / [`set_instance_id(Option<String>)`](crate::operation::get_library_item::builders::GetLibraryItemFluentBuilder::set_instance_id):<br>required: **true**<br><p>The unique identifier of the Amazon Q Business application environment instance.</p><br>
    ///   - [`library_item_id(impl Into<String>)`](crate::operation::get_library_item::builders::GetLibraryItemFluentBuilder::library_item_id) / [`set_library_item_id(Option<String>)`](crate::operation::get_library_item::builders::GetLibraryItemFluentBuilder::set_library_item_id):<br>required: **true**<br><p>The unique identifier of the library item to retrieve.</p><br>
    ///   - [`app_id(impl Into<String>)`](crate::operation::get_library_item::builders::GetLibraryItemFluentBuilder::app_id) / [`set_app_id(Option<String>)`](crate::operation::get_library_item::builders::GetLibraryItemFluentBuilder::set_app_id):<br>required: **false**<br><p>The unique identifier of the Amazon Q App associated with the library item.</p><br>
    /// - On success, responds with [`GetLibraryItemOutput`](crate::operation::get_library_item::GetLibraryItemOutput) with field(s):
    ///   - [`library_item_id(String)`](crate::operation::get_library_item::GetLibraryItemOutput::library_item_id): <p>The unique identifier of the library item.</p>
    ///   - [`app_id(String)`](crate::operation::get_library_item::GetLibraryItemOutput::app_id): <p>The unique identifier of the Q App associated with the library item.</p>
    ///   - [`app_version(i32)`](crate::operation::get_library_item::GetLibraryItemOutput::app_version): <p>The version of the Q App associated with the library item.</p>
    ///   - [`categories(Vec::<Category>)`](crate::operation::get_library_item::GetLibraryItemOutput::categories): <p>The categories associated with the library item for discovery.</p>
    ///   - [`status(String)`](crate::operation::get_library_item::GetLibraryItemOutput::status): <p>The status of the library item, such as "Published".</p>
    ///   - [`created_at(DateTime)`](crate::operation::get_library_item::GetLibraryItemOutput::created_at): <p>The date and time the library item was created.</p>
    ///   - [`created_by(String)`](crate::operation::get_library_item::GetLibraryItemOutput::created_by): <p>The user who created the library item.</p>
    ///   - [`updated_at(Option<DateTime>)`](crate::operation::get_library_item::GetLibraryItemOutput::updated_at): <p>The date and time the library item was last updated.</p>
    ///   - [`updated_by(Option<String>)`](crate::operation::get_library_item::GetLibraryItemOutput::updated_by): <p>The user who last updated the library item.</p>
    ///   - [`rating_count(i32)`](crate::operation::get_library_item::GetLibraryItemOutput::rating_count): <p>The number of ratings the library item has received from users.</p>
    ///   - [`is_rated_by_user(Option<bool>)`](crate::operation::get_library_item::GetLibraryItemOutput::is_rated_by_user): <p>Whether the current user has rated the library item.</p>
    ///   - [`user_count(Option<i32>)`](crate::operation::get_library_item::GetLibraryItemOutput::user_count): <p>The number of users who have associated the Q App with their account.</p>
    ///   - [`is_verified(Option<bool>)`](crate::operation::get_library_item::GetLibraryItemOutput::is_verified): <p>Indicates whether the library item has been verified.</p>
    /// - On failure, responds with [`SdkError<GetLibraryItemError>`](crate::operation::get_library_item::GetLibraryItemError)
    pub fn get_library_item(&self) -> crate::operation::get_library_item::builders::GetLibraryItemFluentBuilder {
        crate::operation::get_library_item::builders::GetLibraryItemFluentBuilder::new(self.handle.clone())
    }
}
