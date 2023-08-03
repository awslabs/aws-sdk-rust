/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::Client;
use aws_smithy_client::test_connection::{capture_request, CaptureRequestReceiver};
use aws_types::SdkConfig;

// TODO(enableNewSmithyRuntimeCleanup): Remove this attribute once #[cfg(aws_sdk_middleware_mode)]
//  has been removed
#[allow(dead_code)]
fn test_client() -> (CaptureRequestReceiver, Client) {
    let (conn, captured_request) = capture_request(None);
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::new("us-west-2"))
        .http_connector(conn)
        .build();
    let client = Client::new(&sdk_config);
    (captured_request, client)
}

#[cfg(not(aws_sdk_middleware_mode))]
#[tokio::test]
async fn operation_overrides_force_path_style() {
    let (captured_request, client) = test_client();
    let _ = client
        .list_objects_v2()
        .bucket("test-bucket")
        .customize()
        .await
        .unwrap()
        .config_override(aws_sdk_s3::config::Config::builder().force_path_style(true))
        .send()
        .await;
    assert_eq!(
        captured_request.expect_request().uri().to_string(),
        "https://s3.us-west-2.amazonaws.com/test-bucket/?list-type=2"
    );
}

#[cfg(not(aws_sdk_middleware_mode))]
#[tokio::test]
async fn operation_overrides_fips() {
    let (captured_request, client) = test_client();
    let _ = client
        .list_objects_v2()
        .bucket("test-bucket")
        .customize()
        .await
        .unwrap()
        .config_override(aws_sdk_s3::config::Config::builder().use_fips(true))
        .send()
        .await;
    assert_eq!(
        captured_request.expect_request().uri().to_string(),
        "https://test-bucket.s3-fips.us-west-2.amazonaws.com/?list-type=2"
    );
}

#[cfg(not(aws_sdk_middleware_mode))]
#[tokio::test]
async fn operation_overrides_dual_stack() {
    let (captured_request, client) = test_client();
    let _ = client
        .list_objects_v2()
        .bucket("test-bucket")
        .customize()
        .await
        .unwrap()
        .config_override(aws_sdk_s3::config::Config::builder().use_dual_stack(true))
        .send()
        .await;
    assert_eq!(
        captured_request.expect_request().uri().to_string(),
        "https://test-bucket.s3.dualstack.us-west-2.amazonaws.com/?list-type=2"
    );
}

// TODO(enableNewSmithyRuntimeCleanup): Comment in the following test once Handle is no longer
//  accessed in ServiceRuntimePlugin::config. Currently, a credentials cache created for a single
//  operation invocation is not picked up by an identity resolver.
/*
#[cfg(not(aws_sdk_middleware_mode))]
#[tokio::test]
async fn operation_overrides_credentials_provider() {
    let (captured_request, client) = test_client();
    let _ = client
        .list_objects_v2()
        .bucket("test-bucket")
        .customize()
        .await
        .unwrap()
        .config_override(aws_sdk_s3::config::Config::builder().credentials_provider(Credentials::new(
            "test",
            "test",
            Some("test".into()),
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1669257290 + 3600)),
            "test",
        )))
        .request_time_for_tests(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1669257290))
        .send()
        .await;

    let request = captured_request.expect_request();
    let actual_auth =
        std::str::from_utf8(request.headers().get("authorization").unwrap().as_bytes()).unwrap();
    // signature would be f98cc3911dfba0daabf4343152f456bff9ecd3888a3068a1346d26949cb8f9e5
    // if we used `Credentials::for_tests()`
    let expected_sig = "Signature=d7e7be63efc37c5bab5eda121999cd1c9a95efdde0cc1ce7c1b8761051cc3cbd";
    assert!(
        actual_auth.contains(expected_sig),
        "authorization header signature did not match expected signature: expected {} but not found in {}",
        expected_sig,
        actual_auth,
    );
}
*/
