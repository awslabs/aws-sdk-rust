/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::http::HttpHeaders;
use aws_smithy_http::operation;
use aws_smithy_http::result::SdkError;
use aws_smithy_types::error::metadata::{
    Builder as ErrorMetadataBuilder, ErrorMetadata, ProvideErrorMetadata,
};
use aws_smithy_types::error::Unhandled;
use http::{HeaderMap, HeaderValue};

/// Constant for the [`ErrorMetadata`] extra field that contains the request ID
const AWS_REQUEST_ID: &str = "aws_request_id";

/// Implementers add a function to return an AWS request ID
pub trait RequestId {
    /// Returns the request ID, or `None` if the service could not be reached.
    fn request_id(&self) -> Option<&str>;
}

impl<E, R> RequestId for SdkError<E, R>
where
    R: HttpHeaders,
{
    fn request_id(&self) -> Option<&str> {
        match self {
            Self::ResponseError(err) => extract_request_id(err.raw().http_headers()),
            Self::ServiceError(err) => extract_request_id(err.raw().http_headers()),
            _ => None,
        }
    }
}

impl RequestId for ErrorMetadata {
    fn request_id(&self) -> Option<&str> {
        self.extra(AWS_REQUEST_ID)
    }
}

impl RequestId for Unhandled {
    fn request_id(&self) -> Option<&str> {
        self.meta().request_id()
    }
}

impl RequestId for operation::Response {
    fn request_id(&self) -> Option<&str> {
        extract_request_id(self.http().headers())
    }
}

impl<B> RequestId for http::Response<B> {
    fn request_id(&self) -> Option<&str> {
        extract_request_id(self.headers())
    }
}

impl<O, E> RequestId for Result<O, E>
where
    O: RequestId,
    E: RequestId,
{
    fn request_id(&self) -> Option<&str> {
        match self {
            Ok(ok) => ok.request_id(),
            Err(err) => err.request_id(),
        }
    }
}

/// Applies a request ID to a generic error builder
#[doc(hidden)]
pub fn apply_request_id(
    builder: ErrorMetadataBuilder,
    headers: &HeaderMap<HeaderValue>,
) -> ErrorMetadataBuilder {
    if let Some(request_id) = extract_request_id(headers) {
        builder.custom(AWS_REQUEST_ID, request_id)
    } else {
        builder
    }
}

/// Extracts a request ID from HTTP response headers
fn extract_request_id(headers: &HeaderMap<HeaderValue>) -> Option<&str> {
    headers
        .get("x-amzn-requestid")
        .or_else(|| headers.get("x-amz-request-id"))
        .and_then(|value| value.to_str().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_http::body::SdkBody;
    use http::Response;

    #[test]
    fn test_request_id_sdk_error() {
        let without_request_id =
            || operation::Response::new(Response::builder().body(SdkBody::empty()).unwrap());
        let with_request_id = || {
            operation::Response::new(
                Response::builder()
                    .header(
                        "x-amzn-requestid",
                        HeaderValue::from_static("some-request-id"),
                    )
                    .body(SdkBody::empty())
                    .unwrap(),
            )
        };
        assert_eq!(
            None,
            SdkError::<(), _>::response_error("test", without_request_id()).request_id()
        );
        assert_eq!(
            Some("some-request-id"),
            SdkError::<(), _>::response_error("test", with_request_id()).request_id()
        );
        assert_eq!(
            None,
            SdkError::service_error((), without_request_id()).request_id()
        );
        assert_eq!(
            Some("some-request-id"),
            SdkError::service_error((), with_request_id()).request_id()
        );
    }

    #[test]
    fn test_extract_request_id() {
        let mut headers = HeaderMap::new();
        assert_eq!(None, extract_request_id(&headers));

        headers.append(
            "x-amzn-requestid",
            HeaderValue::from_static("some-request-id"),
        );
        assert_eq!(Some("some-request-id"), extract_request_id(&headers));

        headers.append(
            "x-amz-request-id",
            HeaderValue::from_static("other-request-id"),
        );
        assert_eq!(Some("some-request-id"), extract_request_id(&headers));

        headers.remove("x-amzn-requestid");
        assert_eq!(Some("other-request-id"), extract_request_id(&headers));
    }

    #[test]
    fn test_apply_request_id() {
        let mut headers = HeaderMap::new();
        assert_eq!(
            ErrorMetadata::builder().build(),
            apply_request_id(ErrorMetadata::builder(), &headers).build(),
        );

        headers.append(
            "x-amzn-requestid",
            HeaderValue::from_static("some-request-id"),
        );
        assert_eq!(
            ErrorMetadata::builder()
                .custom(AWS_REQUEST_ID, "some-request-id")
                .build(),
            apply_request_id(ErrorMetadata::builder(), &headers).build(),
        );
    }

    #[test]
    fn test_error_metadata_request_id_impl() {
        let err = ErrorMetadata::builder()
            .custom(AWS_REQUEST_ID, "some-request-id")
            .build();
        assert_eq!(Some("some-request-id"), err.request_id());
    }
}
