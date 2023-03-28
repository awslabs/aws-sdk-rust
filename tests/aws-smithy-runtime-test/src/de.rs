/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime::{BoxError, ResponseDeserializer};
use aws_smithy_runtime_api::config_bag::ConfigBag;
use aws_smithy_runtime_api::runtime_plugin::RuntimePlugin;

#[derive(Debug)]
pub struct GetObjectResponseDeserializer {}

impl GetObjectResponseDeserializer {
    pub fn new() -> Self {
        Self {}
    }
}

impl RuntimePlugin for GetObjectResponseDeserializer {
    fn configure(&self, _cfg: &mut ConfigBag) -> Result<(), BoxError> {
        todo!()
    }
}

impl ResponseDeserializer<http::Response<SdkBody>, GetObjectOutput>
    for GetObjectResponseDeserializer
{
    fn deserialize_response(
        &self,
        _res: &mut http::Response<SdkBody>,
        _cfg: &ConfigBag,
    ) -> Result<GetObjectOutput, BoxError> {
        todo!()
        // Ok({
        //     #[allow(unused_mut)]
        //     let mut output = aws_sdk_s3::output::get_object_output::Builder::default();
        //     let _ = res;
        //     output = output.set_accept_ranges(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_accept_ranges(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse AcceptRanges from header `accept-ranges",
        //             )
        //         })?,
        //     );
        //     output = output.set_body(Some(
        //         aws_sdk_s3::http_serde::deser_payload_get_object_get_object_output_body(
        //             res.body_mut(),
        //         )?,
        //     ));
        //     output = output.set_bucket_key_enabled(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_bucket_key_enabled(res.headers())
        //             .map_err(|_| GetObjectError::unhandled("Failed to parse BucketKeyEnabled from header `x-amz-server-side-encryption-bucket-key-enabled"))?
        //     );
        //     output = output.set_cache_control(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_cache_control(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse CacheControl from header `Cache-Control",
        //             )
        //         })?,
        //     );
        //     output = output.set_checksum_crc32(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_checksum_crc32(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse ChecksumCRC32 from header `x-amz-checksum-crc32",
        //             )
        //         })?,
        //     );
        //     output = output.set_checksum_crc32_c(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_checksum_crc32_c(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse ChecksumCRC32C from header `x-amz-checksum-crc32c",
        //             )
        //         })?,
        //     );
        //     output = output.set_checksum_sha1(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_checksum_sha1(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse ChecksumSHA1 from header `x-amz-checksum-sha1",
        //             )
        //         })?,
        //     );
        //     output = output.set_checksum_sha256(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_checksum_sha256(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse ChecksumSHA256 from header `x-amz-checksum-sha256",
        //             )
        //         })?,
        //     );
        //     output = output.set_content_disposition(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_content_disposition(
        //             res.headers(),
        //         )
        //             .map_err(|_| {
        //                 GetObjectError::unhandled(
        //                     "Failed to parse ContentDisposition from header `Content-Disposition",
        //                 )
        //             })?,
        //     );
        //     output = output.set_content_encoding(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_content_encoding(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse ContentEncoding from header `Content-Encoding",
        //             )
        //         })?,
        //     );
        //     output = output.set_content_language(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_content_language(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse ContentLanguage from header `Content-Language",
        //             )
        //         })?,
        //     );
        //     output = output.set_content_length(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_content_length(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse ContentLength from header `Content-Length",
        //             )
        //         })?,
        //     );
        //     output = output.set_content_range(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_content_range(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse ContentRange from header `Content-Range",
        //             )
        //         })?,
        //     );
        //     output = output.set_content_type(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_content_type(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse ContentType from header `Content-Type",
        //             )
        //         })?,
        //     );
        //     output = output.set_delete_marker(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_delete_marker(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse DeleteMarker from header `x-amz-delete-marker",
        //             )
        //         })?,
        //     );
        //     output = output.set_e_tag(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_e_tag(
        //             res.headers(),
        //         )
        //         .map_err(|_| GetObjectError::unhandled("Failed to parse ETag from header `ETag"))?,
        //     );
        //     output = output.set_expiration(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_expiration(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse Expiration from header `x-amz-expiration",
        //             )
        //         })?,
        //     );
        //     output = output.set_expires(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_expires(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled("Failed to parse Expires from header `Expires")
        //         })?,
        //     );
        //     output = output.set_last_modified(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_last_modified(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse LastModified from header `Last-Modified",
        //             )
        //         })?,
        //     );
        //     output = output.set_metadata(
        //         aws_sdk_s3::http_serde::deser_prefix_header_get_object_get_object_output_metadata(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse Metadata from prefix header `x-amz-meta-",
        //             )
        //         })?,
        //     );
        //     output = output.set_missing_meta(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_missing_meta(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse MissingMeta from header `x-amz-missing-meta",
        //             )
        //         })?,
        //     );
        //     output = output.set_object_lock_legal_hold_status(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_object_lock_legal_hold_status(res.headers())
        //             .map_err(|_| GetObjectError::unhandled("Failed to parse ObjectLockLegalHoldStatus from header `x-amz-object-lock-legal-hold"))?
        //     );
        //     output = output.set_object_lock_mode(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_object_lock_mode(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse ObjectLockMode from header `x-amz-object-lock-mode",
        //             )
        //         })?,
        //     );
        //     output = output.set_object_lock_retain_until_date(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_object_lock_retain_until_date(res.headers())
        //             .map_err(|_| GetObjectError::unhandled("Failed to parse ObjectLockRetainUntilDate from header `x-amz-object-lock-retain-until-date"))?
        //     );
        //     output = output.set_parts_count(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_parts_count(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse PartsCount from header `x-amz-mp-parts-count",
        //             )
        //         })?,
        //     );
        //     output = output.set_replication_status(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_replication_status(
        //             res.headers(),
        //         )
        //             .map_err(|_| {
        //                 GetObjectError::unhandled(
        //                     "Failed to parse ReplicationStatus from header `x-amz-replication-status",
        //                 )
        //             })?,
        //     );
        //     output = output.set_request_charged(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_request_charged(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse RequestCharged from header `x-amz-request-charged",
        //             )
        //         })?,
        //     );
        //     output = output.set_restore(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_restore(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled("Failed to parse Restore from header `x-amz-restore")
        //         })?,
        //     );
        //     output = output.set_sse_customer_algorithm(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_sse_customer_algorithm(res.headers())
        //             .map_err(|_| GetObjectError::unhandled("Failed to parse SSECustomerAlgorithm from header `x-amz-server-side-encryption-customer-algorithm"))?
        //     );
        //     output = output.set_sse_customer_key_md5(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_sse_customer_key_md5(res.headers())
        //             .map_err(|_| GetObjectError::unhandled("Failed to parse SSECustomerKeyMD5 from header `x-amz-server-side-encryption-customer-key-MD5"))?
        //     );
        //     output = output.set_ssekms_key_id(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_ssekms_key_id(res.headers())
        //             .map_err(|_| GetObjectError::unhandled("Failed to parse SSEKMSKeyId from header `x-amz-server-side-encryption-aws-kms-key-id"))?
        //     );
        //     output = output.set_server_side_encryption(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_server_side_encryption(res.headers())
        //             .map_err(|_| GetObjectError::unhandled("Failed to parse ServerSideEncryption from header `x-amz-server-side-encryption"))?
        //     );
        //     output = output.set_storage_class(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_storage_class(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse StorageClass from header `x-amz-storage-class",
        //             )
        //         })?,
        //     );
        //     output = output.set_tag_count(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_tag_count(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse TagCount from header `x-amz-tagging-count",
        //             )
        //         })?,
        //     );
        //     output = output.set_version_id(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_version_id(
        //             res.headers(),
        //         )
        //         .map_err(|_| {
        //             GetObjectError::unhandled(
        //                 "Failed to parse VersionId from header `x-amz-version-id",
        //             )
        //         })?,
        //     );
        //     output = output.set_website_redirect_location(
        //         aws_sdk_s3::http_serde::deser_header_get_object_get_object_output_website_redirect_location(res.headers())
        //             .map_err(|_| GetObjectError::unhandled("Failed to parse WebsiteRedirectLocation from header `x-amz-website-redirect-location"))?
        //     );
        //     output._set_extended_request_id(
        //         aws_sdk_s3::s3_request_id::RequestIdExt::extended_request_id(res)
        //             .map(str::to_string),
        //     );
        //     output._set_request_id(
        //         aws_http::request_id::RequestId::request_id(res).map(str::to_string),
        //     );
        //     let response_algorithms = ["crc32", "crc32c", "sha256", "sha1"].as_slice();
        //     let checksum_mode = cfg.get::<aws_sdk_s3::model::ChecksumMode>();
        //     // Per [the spec](https://awslabs.github.io/smithy/1.0/spec/aws/aws-core.html#http-response-checksums),
        //     // we check to see if it's the `ENABLED` variant
        //     if matches!(
        //         checksum_mode,
        //         Some(&aws_sdk_s3::model::ChecksumMode::Enabled)
        //     ) {
        //         if let Some((checksum_algorithm, precalculated_checksum)) =
        //             aws_sdk_s3::http_body_checksum::check_headers_for_precalculated_checksum(
        //                 res.headers(),
        //                 response_algorithms,
        //             )
        //         {
        //             let bytestream = output.body.take().map(|bytestream| {
        //                 bytestream.map(move |sdk_body| {
        //                     aws_sdk_s3::http_body_checksum::wrap_body_with_checksum_validator(
        //                         sdk_body,
        //                         checksum_algorithm,
        //                         precalculated_checksum.clone(),
        //                     )
        //                 })
        //             });
        //             output = output.set_body(bytestream);
        //         }
        //     }
        //     output.build()
        // })
    }
}
