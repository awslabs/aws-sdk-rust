/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::model::{
    CompressionType, CsvInput, CsvOutput, ExpressionType, FileHeaderInfo, InputSerialization,
    OutputSerialization,
};
use aws_sdk_s3::{Client, Config, Credentials, Region};
use aws_smithy_async::assert_elapsed;
use aws_smithy_async::rt::sleep::{AsyncSleep, TokioSleep};
use aws_smithy_client::never::NeverService;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
use aws_smithy_types::timeout;
use aws_smithy_types::tristate::TriState;

use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_timeout_service_ends_request_that_never_completes() {
    let conn: NeverService<http::Request<SdkBody>, http::Response<SdkBody>, ConnectorError> =
        NeverService::new();
    let region = Region::from_static("us-east-2");
    let credentials = Credentials::new("test", "test", None, None, "test");
    let api_timeouts =
        timeout::Api::new().with_call_timeout(TriState::Set(Duration::from_secs_f32(0.5)));
    let timeout_config = timeout::Config::new().with_api_timeouts(api_timeouts);
    let sleep_impl: Arc<dyn AsyncSleep> = Arc::new(TokioSleep::new());
    let config = Config::builder()
        .region(region)
        .credentials_provider(credentials)
        .timeout_config(timeout_config)
        .sleep_impl(sleep_impl)
        .build();
    let client = Client::from_conf_conn(config, conn.clone());

    let now = tokio::time::Instant::now();
    tokio::time::pause();

    let err = client
        .select_object_content()
        .bucket("aws-rust-sdk")
        .key("sample_data.csv")
        .expression_type(ExpressionType::Sql)
        .expression("SELECT * FROM s3object s WHERE s.\"Name\" = 'Jane'")
        .input_serialization(
            InputSerialization::builder()
                .csv(
                    CsvInput::builder()
                        .file_header_info(FileHeaderInfo::Use)
                        .build(),
                )
                .compression_type(CompressionType::None)
                .build(),
        )
        .output_serialization(
            OutputSerialization::builder()
                .csv(CsvOutput::builder().build())
                .build(),
        )
        .send()
        .await
        .unwrap_err();

    assert_eq!(format!("{:?}", err), "TimeoutError(RequestTimeoutError { kind: \"API call (all attempts including retries)\", duration: 500ms })");
    assert_elapsed!(now, std::time::Duration::from_secs_f32(0.5));
}
