// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetTile`](crate::operation::get_tile::builders::GetTileFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`tileset(impl Into<String>)`](crate::operation::get_tile::builders::GetTileFluentBuilder::tileset) / [`set_tileset(Option<String>)`](crate::operation::get_tile::builders::GetTileFluentBuilder::set_tileset):<br>required: **true**<br><p>Specifies the desired tile set.</p> <p>Valid Values: <code>raster.satellite | vector.basemap</code></p><br>
    ///   - [`z(impl Into<String>)`](crate::operation::get_tile::builders::GetTileFluentBuilder::z) / [`set_z(Option<String>)`](crate::operation::get_tile::builders::GetTileFluentBuilder::set_z):<br>required: **true**<br><p>The zoom value for the map tile.</p><br>
    ///   - [`x(impl Into<String>)`](crate::operation::get_tile::builders::GetTileFluentBuilder::x) / [`set_x(Option<String>)`](crate::operation::get_tile::builders::GetTileFluentBuilder::set_x):<br>required: **true**<br><p>The X axis value for the map tile. Must be between 0 and 19.</p><br>
    ///   - [`y(impl Into<String>)`](crate::operation::get_tile::builders::GetTileFluentBuilder::y) / [`set_y(Option<String>)`](crate::operation::get_tile::builders::GetTileFluentBuilder::set_y):<br>required: **true**<br><p>The Y axis value for the map tile.</p><br>
    ///   - [`key(impl Into<String>)`](crate::operation::get_tile::builders::GetTileFluentBuilder::key) / [`set_key(Option<String>)`](crate::operation::get_tile::builders::GetTileFluentBuilder::set_key):<br>required: **false**<br><p>Optional: The API key to be used for authorization. Either an API key or valid SigV4 signature must be provided when making a request.</p><br>
    /// - On success, responds with [`GetTileOutput`](crate::operation::get_tile::GetTileOutput) with field(s):
    ///   - [`blob(Option<Blob>)`](crate::operation::get_tile::GetTileOutput::blob): <p>The blob represents a vector tile in <code>mvt</code> or a raster tile in an image format.</p>
    ///   - [`content_type(Option<String>)`](crate::operation::get_tile::GetTileOutput::content_type): <p>Header that represents the format of the response. The response returns the following as the HTTP body.</p>
    ///   - [`cache_control(Option<String>)`](crate::operation::get_tile::GetTileOutput::cache_control): <p>Header that instructs caching configuration for the client.</p>
    ///   - [`e_tag(Option<String>)`](crate::operation::get_tile::GetTileOutput::e_tag): <p>The pricing bucket for which the request is charged at.</p>
    ///   - [`pricing_bucket(String)`](crate::operation::get_tile::GetTileOutput::pricing_bucket): <p>The pricing bucket for which the request is charged at.</p>
    /// - On failure, responds with [`SdkError<GetTileError>`](crate::operation::get_tile::GetTileError)
    pub fn get_tile(&self) -> crate::operation::get_tile::builders::GetTileFluentBuilder {
        crate::operation::get_tile::builders::GetTileFluentBuilder::new(self.handle.clone())
    }
}
