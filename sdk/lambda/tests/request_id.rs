/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_lambda::config::{Credentials, Region};
use aws_sdk_lambda::operation::list_functions::ListFunctionsError;
use aws_sdk_lambda::operation::RequestId;
use aws_sdk_lambda::{Client, Config};
use aws_smithy_client::test_connection::infallible_connection_fn;

async fn run_test(
    response: impl Fn() -> http::Response<&'static str> + Send + Sync + 'static,
    expect_error: bool,
) {
    let conn = infallible_connection_fn(move |_| response());
    let conf = Config::builder()
        .http_connector(conn)
        .credentials_provider(Credentials::for_tests())
        .region(Region::from_static("us-east-1"))
        .build();
    let client = Client::from_conf(conf);
    let resp = client.list_functions().send().await;
    if expect_error {
        let err = resp.err().expect("should be an error").into_service_error();
        assert!(matches!(err, ListFunctionsError::Unhandled(_)));
        assert_eq!(Some("correct-request-id"), err.request_id());
        assert_eq!(Some("correct-request-id"), err.meta().request_id());
    } else {
        let output = resp.expect("should be successful");
        assert_eq!(Some("correct-request-id"), output.request_id());
    }
}

#[tokio::test]
async fn get_request_id_from_unmodeled_error() {
    run_test(
        || {
            http::Response::builder()
                .header("x-amzn-RequestId", "correct-request-id")
                .header("X-Amzn-Errortype", "ListFunctions")
                .status(500)
                .body("{}")
                .unwrap()
        },
        true,
    )
    .await;
}

#[tokio::test]
async fn get_request_id_from_successful_response() {
    run_test(
        || {
            http::Response::builder()
                .header("x-amzn-RequestId", "correct-request-id")
                .status(200)
                .body(r#"{"Functions":[],"NextMarker":null}"#)
                .unwrap()
        },
        false,
    )
    .await;
}
