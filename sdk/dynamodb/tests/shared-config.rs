/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_runtime::env_config::file::{EnvConfigFileKind, EnvConfigFiles};
use aws_sdk_dynamodb::config::{
    BehaviorVersion, Credentials, Region, StalledStreamProtectionConfig,
};
use aws_smithy_runtime::client::http::test_util::capture_request;
use http::Uri;

/// Iterative test of loading clients from shared configuration
#[tokio::test]
async fn shared_config_testbed() {
    let shared_config = aws_types::SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .build();
    let (http_client, request) = capture_request(None);
    let conf = aws_sdk_dynamodb::config::Builder::from(&shared_config)
        .credentials_provider(Credentials::for_tests())
        .http_client(http_client)
        .endpoint_url("http://localhost:8000")
        .build();
    let svc = aws_sdk_dynamodb::Client::from_conf(conf);
    let _ = svc.list_tables().send().await;
    assert_eq!(
        request.expect_request().uri(),
        &Uri::from_static("http://localhost:8000")
    );
}

#[tokio::test]
async fn service_config_from_profile() {
    let _ = tracing_subscriber::fmt::try_init();

    let config = r#"
[profile custom]
aws_access_key_id = test-access-key-id
aws_secret_access_key = test-secret-access-key
aws_session_token = test-session-token
region = us-east-1
services = custom

[services custom]
dynamodb =
  region = us-west-1
"#
    .trim();

    let shared_config = aws_config::ConfigLoader::default()
        .behavior_version(BehaviorVersion::latest())
        .profile_name("custom")
        .profile_files(
            EnvConfigFiles::builder()
                .with_contents(EnvConfigFileKind::Config, config)
                .build(),
        )
        .load()
        .await;
    let service_config = aws_sdk_dynamodb::Config::from(&shared_config);

    assert_eq!(
        service_config.region().unwrap(),
        &Region::from_static("us-west-1")
    );
}
