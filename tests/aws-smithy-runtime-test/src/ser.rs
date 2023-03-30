/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::operation::get_object::GetObjectInput;
use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime::{BoxError, RequestSerializer};
use aws_smithy_runtime_api::config_bag::ConfigBag;
use aws_smithy_runtime_api::runtime_plugin::RuntimePlugin;

#[derive(Debug)]
pub struct GetObjectInputSerializer {}

impl GetObjectInputSerializer {
    pub fn new() -> Self {
        Self {}
    }
}

impl RuntimePlugin for GetObjectInputSerializer {
    fn configure(&self, _cfg: &mut ConfigBag) -> Result<(), BoxError> {
        todo!()
    }
}

impl RequestSerializer<GetObjectInput, http::Request<SdkBody>> for GetObjectInputSerializer {
    fn serialize_request(
        &self,
        _input: &mut GetObjectInput,
        _cfg: &ConfigBag,
    ) -> Result<http::Request<SdkBody>, BoxError> {
        todo!()
        // let request = {
        //     fn uri_base(_input: &GetObjectInput, output: &mut String) -> Result<(), BuildError> {
        //         use std::fmt::Write;
        //
        //         let input_30 = &_input.key;
        //         let input_30 = input_30
        //             .as_ref()
        //             .ok_or_else(|| BuildError::missing_field("key", "cannot be empty or unset"))?;
        //         let key = aws_smithy_http::label::fmt_string(
        //             input_30,
        //             aws_smithy_http::label::EncodingStrategy::Greedy,
        //         );
        //         if key.is_empty() {
        //             return Err(BuildError::missing_field("key", "cannot be empty or unset"));
        //         }
        //         write!(output, "/{Key}", Key = key).expect("formatting should succeed");
        //         Ok(())
        //     }
        //     fn uri_query(
        //         _input: &GetObjectInput,
        //         mut output: &mut String,
        //     ) -> Result<(), BuildError> {
        //         let mut query = aws_smithy_http::query::Writer::new(&mut output);
        //         query.push_kv("x-id", "GetObject");
        //         if let Some(inner_31) = &_input.response_cache_control {
        //             {
        //                 query.push_kv(
        //                     "response-cache-control",
        //                     &aws_smithy_http::query::fmt_string(&inner_31),
        //                 );
        //             }
        //         }
        //         if let Some(inner_32) = &_input.response_content_disposition {
        //             {
        //                 query.push_kv(
        //                     "response-content-disposition",
        //                     &aws_smithy_http::query::fmt_string(&inner_32),
        //                 );
        //             }
        //         }
        //         if let Some(inner_33) = &_input.response_content_encoding {
        //             {
        //                 query.push_kv(
        //                     "response-content-encoding",
        //                     &aws_smithy_http::query::fmt_string(&inner_33),
        //                 );
        //             }
        //         }
        //         if let Some(inner_34) = &_input.response_content_language {
        //             {
        //                 query.push_kv(
        //                     "response-content-language",
        //                     &aws_smithy_http::query::fmt_string(&inner_34),
        //                 );
        //             }
        //         }
        //         if let Some(inner_35) = &_input.response_content_type {
        //             {
        //                 query.push_kv(
        //                     "response-content-type",
        //                     &aws_smithy_http::query::fmt_string(&inner_35),
        //                 );
        //             }
        //         }
        //         if let Some(inner_36) = &_input.response_expires {
        //             {
        //                 query.push_kv(
        //                     "response-expires",
        //                     &aws_smithy_http::query::fmt_timestamp(
        //                         inner_36,
        //                         aws_smithy_types::date_time::Format::HttpDate,
        //                     )?,
        //                 );
        //             }
        //         }
        //         if let Some(inner_37) = &_input.version_id {
        //             {
        //                 query.push_kv("versionId", &aws_smithy_http::query::fmt_string(&inner_37));
        //             }
        //         }
        //         if let Some(inner_38) = &_input.part_number {
        //             if *inner_38 != 0 {
        //                 query.push_kv(
        //                     "partNumber",
        //                     aws_smithy_types::primitive::Encoder::from(*inner_38).encode(),
        //                 );
        //             }
        //         }
        //         Ok(())
        //     }
        //
        //     fn update_http_builder(
        //         input: &GetObjectInput,
        //         builder: http::request::Builder,
        //     ) -> Result<http::request::Builder, BuildError> {
        //         let mut uri = String::new();
        //         uri_base(input, &mut uri)?;
        //         uri_query(input, &mut uri)?;
        //         let builder = aws_sdk_s3::http_serde::add_headers_get_object(input, builder)?;
        //         Ok(builder.method("GET").uri(uri))
        //     }
        //     let builder = update_http_builder(&input, http::request::Builder::new())?;
        //     builder
        // };
        //
        // let _properties = aws_smithy_http::property_bag::SharedPropertyBag::new();
        // #[allow(clippy::useless_conversion)]
        // let body = aws_smithy_http::body::SdkBody::from("");
        // Ok(request.body(body).expect("should be valid request"))
    }
}
