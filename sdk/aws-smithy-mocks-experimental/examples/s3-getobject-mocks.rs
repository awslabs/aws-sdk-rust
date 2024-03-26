/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Example of mocking a simple wrapper around S3

#[tokio::main]
async fn main() {
    // this is an example of writing tests, see the tests
}

// TODO(fix-aws-smithy-mocks-experimental) This is currently broken because it
//     depends on a generated crate examples must have a `main()` fn so it was
//     easier to comment out the code below rather than cfg-gate it.
// use aws_sdk_s3::operation::get_object::GetObjectError;
// use aws_sdk_s3::Client;
// use std::error::Error;
//
// pub struct MyFileRetriever {
//     s3_client: Client,
// }
//
// impl MyFileRetriever {
//     pub async fn get_file(&self, path: &str) -> Result<Option<String>, Box<dyn Error>> {
//         let response = match self
//             .s3_client
//             .get_object()
//             .bucket("test-bucket")
//             .key(path)
//             .send()
//             .await
//             .map_err(|e| e.into_service_error())
//         {
//             Ok(response) => response,
//             Err(GetObjectError::NoSuchKey(_)) => return Ok(None),
//             e @ Err(_) => e?,
//         };
//         let contents = response.body.collect().await?.to_vec();
//         let contents = String::from_utf8(contents)?;
//         Ok(Some(contents))
//     }
// }
//
// // intentionally not cfg(test) so that rustdoc can find this
// mod test {
//     use aws_sdk_s3::config::Region;
//     use aws_sdk_s3::operation::get_object::{GetObjectError, GetObjectOutput};
//     use aws_sdk_s3::types::error::NoSuchKey;
//     use aws_sdk_s3::Client;
//     use aws_smithy_mocks_experimental::{mock, MockResponseInterceptor};
//     use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
//     use aws_smithy_runtime_api::http::StatusCode;
//     use aws_smithy_types::body::SdkBody;
//     use aws_smithy_types::byte_stream::ByteStream;
//
//     #[allow(dead_code)]
//     fn mocked_client(file_contents: impl AsRef<[u8]>) -> Client {
//         let file_contents = file_contents.as_ref().to_vec();
//         let get_object_happy_path = mock!(Client::get_object)
//             .match_requests(|req| {
//                 req.bucket() == Some("test-bucket") && req.key() == Some("test-key")
//             })
//             .then_output(move || {
//                 GetObjectOutput::builder()
//                     .body(ByteStream::from(file_contents.clone()))
//                     .build()
//             });
//         // fallback error
//         let get_object_error_path = mock!(Client::get_object)
//             .then_error(|| GetObjectError::NoSuchKey(NoSuchKey::builder().build()));
//         let hinted_500_error = mock!(Client::get_object)
//             .match_requests(|req| req.key() == Some("500"))
//             .then_http_response(|| {
//                 HttpResponse::new(
//                     StatusCode::try_from(500).unwrap(),
//                     SdkBody::from("internal server error"),
//                 )
//             });
//         let mock_response_interceptor = MockResponseInterceptor::new()
//             .with_rule(&get_object_happy_path)
//             .with_rule(&hinted_500_error)
//             .with_rule(&get_object_error_path);
//         Client::from_conf(
//             aws_sdk_s3::Config::builder()
//                 .with_test_defaults()
//                 .region(Region::from_static("us-east-1"))
//                 .interceptor(mock_response_interceptor)
//                 .build(),
//         )
//     }
//
//     #[tokio::test]
//     async fn loads_file() {
//         let client = super::MyFileRetriever {
//             s3_client: mocked_client(b"12345-abcde"),
//         };
//         assert_eq!(
//             client.get_file("test-key").await.unwrap().as_deref(),
//             Some("12345-abcde")
//         );
//         assert_eq!(client.get_file("different-key").await.unwrap(), None)
//     }
//
//     #[tokio::test]
//     async fn returns_error_on_invalid_utf8() {
//         let client = super::MyFileRetriever {
//             s3_client: mocked_client(&vec![0xFF, 0xFE]),
//         };
//         client
//             .get_file("test-key")
//             .await
//             .expect_err("invalid UTF-8");
//     }
// }
