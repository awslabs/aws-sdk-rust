/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// TODO(fix-aws-smithy-mocks-experimental) This is currently broken because it depends on a generated crate

// use aws_sdk_s3::config::Region;
// use aws_sdk_s3::operation::get_object::{GetObjectError, GetObjectOutput};
// use aws_sdk_s3::operation::list_buckets::ListBucketsError;
// use aws_sdk_s3::{Client, Config};
// use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
// use aws_smithy_runtime_api::http::StatusCode;
// use aws_smithy_types::body::SdkBody;
// use aws_smithy_types::byte_stream::ByteStream;
// use aws_smithy_types::error::metadata::ProvideErrorMetadata;
// use aws_smithy_types::error::ErrorMetadata;
//
// use aws_smithy_mocks_experimental::{mock, mock_client, MockResponseInterceptor, RuleMode};
//
// const S3_NO_SUCH_KEY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
// <Error>
//   <Code>NoSuchKey</Code>
//   <Message>The resource you requested does not exist</Message>
//   <Resource>/mybucket/myfoto.jpg</Resource>
//   <RequestId>4442587FB7D0A2F9</RequestId>
// </Error>"#;
//
// #[tokio::test]
// async fn create_mock_s3_get_object() {
//     let s3_404 = mock!(Client::get_object)
//         .match_requests(|inp| {
//             inp.bucket() == Some("test-bucket") && inp.key() != Some("correct-key")
//         })
//         .then_http_response(|| {
//             HttpResponse::new(
//                 StatusCode::try_from(400).unwrap(),
//                 SdkBody::from(S3_NO_SUCH_KEY),
//             )
//         });
//
//     let s3_real_object = mock!(Client::get_object)
//         .match_requests(|inp| {
//             inp.bucket() == Some("test-bucket") && inp.key() == Some("correct-key")
//         })
//         .then_output(|| {
//             GetObjectOutput::builder()
//                 .body(ByteStream::from_static(b"test-test-test"))
//                 .build()
//         });
//
//     let modeled_error = mock!(Client::list_buckets).then_error(|| {
//         ListBucketsError::generic(ErrorMetadata::builder().code("InvalidAccessKey").build())
//     });
//
//     let get_object_mocks = MockResponseInterceptor::new()
//         .rule_mode(RuleMode::Sequential)
//         .with_rule(&s3_404)
//         .with_rule(&s3_real_object)
//         .with_rule(&modeled_error);
//
//     let s3 = aws_sdk_s3::Client::from_conf(
//         Config::builder()
//             .with_test_defaults()
//             .region(Region::new("us-east-1"))
//             .interceptor(get_object_mocks)
//             .build(),
//     );
//
//     let error = s3
//         .get_object()
//         .bucket("test-bucket")
//         .key("foo")
//         .send()
//         .await
//         .expect_err("404");
//     assert!(matches!(
//         error.into_service_error(),
//         GetObjectError::NoSuchKey(_)
//     ));
//     assert_eq!(s3_404.num_calls(), 1);
//
//     let data = s3
//         .get_object()
//         .bucket("test-bucket")
//         .key("correct-key")
//         .send()
//         .await
//         .expect("success response")
//         .body
//         .collect()
//         .await
//         .expect("successful read")
//         .to_vec();
//     assert_eq!(data, b"test-test-test");
//     assert_eq!(s3_real_object.num_calls(), 1);
//
//     let err = s3.list_buckets().send().await.expect_err("bad access key");
//     assert_eq!(err.code(), Some("InvalidAccessKey"));
// }
//
// #[tokio::test]
// async fn mock_client() {
//     let s3_404 = mock!(Client::get_object).then_http_response(|| {
//         HttpResponse::new(
//             StatusCode::try_from(400).unwrap(),
//             SdkBody::from(S3_NO_SUCH_KEY),
//         )
//     });
//
//     let s3_real_object = mock!(Client::get_object).then_output(|| {
//         GetObjectOutput::builder()
//             .body(ByteStream::from_static(b"test-test-test"))
//             .build()
//     });
//
//     let s3 = mock_client!(aws_sdk_s3, [&s3_404, &s3_real_object]);
//
//     let error = s3
//         .get_object()
//         .bucket("test-bucket")
//         .key("foo")
//         .send()
//         .await
//         .expect_err("404");
//     assert!(matches!(
//         error.into_service_error(),
//         GetObjectError::NoSuchKey(_)
//     ));
//     assert_eq!(s3_404.num_calls(), 1);
//
//     let data = s3
//         .get_object()
//         .bucket("test-bucket")
//         .key("correct-key")
//         .send()
//         .await
//         .expect("success response")
//         .body
//         .collect()
//         .await
//         .expect("successful read")
//         .to_vec();
//     assert_eq!(data, b"test-test-test");
//     assert_eq!(s3_real_object.num_calls(), 1);
// }
