/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::orchestrator::Metadata;
use aws_smithy_runtime_api::http::{Request, Response};
use aws_smithy_schema::protocol::{apply_http_endpoint, ClientProtocolInner};
use aws_smithy_schema::serde::{
    SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer,
};
use aws_smithy_schema::{shape_id, Schema, ShapeId};
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_xml::codec::find_depth2_element_slice_by;

use crate::codec::serializer::QueryShapeSerializer;

#[derive(Debug)]
pub struct AwsQueryProtocol {
    protocol_id: ShapeId,
    service_version: String,
}

impl AwsQueryProtocol {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            protocol_id: shape_id!("aws.protocols", "awsQuery"),
            service_version: version.into(),
        }
    }
}

impl ClientProtocolInner for AwsQueryProtocol {
    type Request = Request;
    type Response = Response;

    fn protocol_id(&self) -> &ShapeId {
        &self.protocol_id
    }

    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<Request, SerdeError> {
        let op_name = cfg
            .load::<Metadata>()
            .map(|m| m.name().to_string())
            .ok_or_else(|| {
                SerdeError::custom(
                    "operation Metadata is required to serialize an awsQuery request (Action=)",
                )
            })?;

        let mut serializer = QueryShapeSerializer::new(&op_name, &self.service_version);
        serializer.write_struct(input_schema, input)?;
        let body = aws_smithy_schema::codec::FinishSerializer::finish(serializer);

        let uri = if endpoint.is_empty() { "/" } else { endpoint };
        let mut request = Request::new(SdkBody::from(body));
        request
            .set_method("POST")
            .map_err(|e| SerdeError::custom(format!("{e}")))?;
        request
            .set_uri(uri)
            .map_err(|e| SerdeError::custom(format!("{e}")))?;
        request
            .headers_mut()
            .insert("Content-Type", "application/x-www-form-urlencoded");
        if let Some(len) = request.body().content_length() {
            request
                .headers_mut()
                .insert("Content-Length", len.to_string());
        }
        Ok(request)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a Response,
        _output_schema: &Schema,
        _cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
        use aws_smithy_schema::codec::Codec;
        use aws_smithy_xml::codec::{XmlCodec, XmlCodecSettings};

        let body = response
            .body()
            .bytes()
            .ok_or_else(|| SerdeError::custom("response body not available"))?;
        let body_str = std::str::from_utf8(body).map_err(|e| SerdeError::InvalidInput {
            message: e.to_string(),
        })?;

        // Strip the AWS Query response envelope down to the `<...Result>` (or
        // `<Error>`) element, inclusive of its tags, so the XML deserializer
        // can treat it as the output struct's root wrapper element (the merged
        // `XmlDeserializer::read_struct` reads members from the root element's
        // children).
        let inner = strip_aws_query_envelope(body_str);

        // AWS Query deserializes responses as XML with a default timestamp
        // format of `date-time` (per the protocol spec). `XmlCodec` is
        // stateless; the returned deserializer borrows `inner` (and therefore
        // `response`), not the local codec — `create_deserializer` clones the
        // shared settings `Arc` into the deserializer.
        let codec = XmlCodec::new(
            XmlCodecSettings::builder()
                .default_timestamp_format(aws_smithy_types::date_time::Format::DateTime)
                .build(),
        );
        Ok(Box::new(codec.create_deserializer(inner.as_bytes())))
    }

    fn update_endpoint(
        &self,
        request: &mut Request,
        endpoint: &aws_smithy_types::endpoint::Endpoint,
        cfg: &ConfigBag,
    ) -> Result<(), SerdeError> {
        apply_http_endpoint(request, endpoint, cfg)
    }
}

/// Extracts the result (or error) element from an AWS Query XML response
/// envelope, returning the element *inclusive of its tags*.
///
/// AWS Query responses are shaped like:
/// ```xml
/// <OperationNameResponse>
///   <OperationNameResult> ... </OperationNameResult>
///   <ResponseMetadata>...</ResponseMetadata>
/// </OperationNameResponse>
/// ```
/// or, for errors:
/// ```xml
/// <ErrorResponse><Error> ... </Error></ErrorResponse>
/// ```
/// We locate the depth-2 element whose local name ends with `Result` or equals
/// `Error` and return its full `<El>...</El>` slice so it can be handed to an
/// `XmlDeserializer` as the output struct's root wrapper element.
///
/// Delegates the depth-2 lookup to `aws_smithy_xml`'s shared
/// [`find_depth2_element_slice_by`] (the same utility the REST XML error path
/// uses). If no such element is found — or the body isn't valid XML — we fall
/// back to the whole body and let the downstream `XmlDeserializer` surface any
/// error, mirroring the REST XML fallback.
fn strip_aws_query_envelope(xml: &str) -> &str {
    match find_depth2_element_slice_by(xml.as_bytes(), |name| {
        name.ends_with("Result") || name == "Error"
    }) {
        // The returned slice is a sub-slice of `xml` bounded by ASCII `<`/`>`,
        // so it is always valid UTF-8.
        Some(slice) => std::str::from_utf8(slice).unwrap_or(xml),
        None => xml,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::protocol::ClientProtocolInner;
    use aws_smithy_schema::serde::ShapeSerializer;
    use aws_smithy_schema::ShapeType;
    use aws_smithy_types::config_bag::Layer;

    struct EmptyInput;
    impl SerializableStruct for EmptyInput {
        fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    static SCHEMA: Schema = Schema::new(shape_id!("test", "Input"), ShapeType::Structure);

    fn cfg_with_metadata() -> ConfigBag {
        let mut layer = Layer::new("test");
        layer.store_put(Metadata::new("GetUser", "MyService"));
        ConfigBag::of_layers(vec![layer])
    }

    #[test]
    fn request_has_correct_content_type() {
        let cfg = cfg_with_metadata();
        let request = AwsQueryProtocol::new("2012-11-05")
            .serialize_request(&EmptyInput, &SCHEMA, "https://example.com", &cfg)
            .unwrap();
        assert_eq!(
            request.headers().get("Content-Type").unwrap(),
            "application/x-www-form-urlencoded"
        );
    }

    #[test]
    fn request_has_action_and_version() {
        let cfg = cfg_with_metadata();
        let request = AwsQueryProtocol::new("2012-11-05")
            .serialize_request(&EmptyInput, &SCHEMA, "https://example.com", &cfg)
            .unwrap();
        let body = std::str::from_utf8(request.body().bytes().unwrap()).unwrap();
        assert!(body.contains("Action=GetUser"));
        assert!(body.contains("Version=2012-11-05"));
    }

    #[test]
    fn request_posts_to_endpoint() {
        let cfg = cfg_with_metadata();
        let request = AwsQueryProtocol::new("1.0")
            .serialize_request(
                &EmptyInput,
                &SCHEMA,
                "https://sqs.us-east-1.amazonaws.com",
                &cfg,
            )
            .unwrap();
        assert_eq!(request.uri(), "https://sqs.us-east-1.amazonaws.com");
    }

    #[test]
    fn request_defaults_to_slash() {
        let cfg = cfg_with_metadata();
        let request = AwsQueryProtocol::new("1.0")
            .serialize_request(&EmptyInput, &SCHEMA, "", &cfg)
            .unwrap();
        assert_eq!(request.uri(), "/");
    }

    #[test]
    fn deserialize_response_strips_envelope() {
        let xml = "<GetUserResponse><GetUserResult><Name>Alice</Name><Age>30</Age></GetUserResult></GetUserResponse>";
        let response = Response::new(200u16.try_into().unwrap(), SdkBody::from(xml));

        static NAME: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::String, "Name", 0);
        static AGE: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::Integer, "Age", 1);
        static OUT_SCHEMA: Schema =
            Schema::new_struct(shape_id!("t", "S"), ShapeType::Structure, &[&NAME, &AGE]);

        let mut deser = AwsQueryProtocol::new("1.0")
            .deserialize_response(&response, &OUT_SCHEMA, &ConfigBag::base())
            .unwrap();
        let mut name = String::new();
        let mut age = 0i32;
        deser
            .read_struct(&OUT_SCHEMA, &mut |member, d| {
                match member.member_name() {
                    Some("Name") => name = d.read_string(member)?,
                    Some("Age") => age = d.read_integer(member)?,
                    _ => {}
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(name, "Alice");
        assert_eq!(age, 30);
    }

    #[test]
    fn strip_envelope_returns_self_closing_result_element() {
        // A self-closing result element (empty output) must still be returned
        // inclusive of its tags, not fall through to the whole-document root.
        let xml = "<GetUserResponse><GetUserResult/><ResponseMetadata><RequestId>r</RequestId></ResponseMetadata></GetUserResponse>";
        assert_eq!(strip_aws_query_envelope(xml), "<GetUserResult/>");
    }

    #[test]
    fn strip_envelope_returns_error_element() {
        let xml = "<ErrorResponse><Error><Code>Boom</Code></Error></ErrorResponse>";
        assert_eq!(
            strip_aws_query_envelope(xml),
            "<Error><Code>Boom</Code></Error>"
        );
    }

    #[test]
    fn protocol_id() {
        assert_eq!(
            AwsQueryProtocol::new("1.0").protocol_id().as_str(),
            "aws.protocols#awsQuery"
        );
    }
}
