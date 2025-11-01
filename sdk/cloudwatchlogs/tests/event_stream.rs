/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::Credentials;
use aws_sdk_cloudwatchlogs::{
    config::Region, operation::start_live_tail::StartLiveTailOutput,
    types::LiveTailSessionMetadata, Client, Config,
};
use aws_smithy_eventstream::test_util::validate_body;
use aws_smithy_runtime::client::http::test_util::dvr::ReplayingClient;

#[ignore] // TODO(re-enable this after success.json has been updated)
#[tokio::test]
async fn operation_with_rpc_bound_protocol() {
    let (replayer, mut output) = start_request("us-west-2", "tests/success.json").await;

    let mut session_metadata: Option<LiveTailSessionMetadata> = None;

    while let Some(event) = output.response_stream.recv().await.unwrap() {
        match event {
            aws_sdk_cloudwatchlogs::types::StartLiveTailResponseStream::SessionStart(_) => {
                // `SessionStart` event has been removed from `success.json` for security reason
            }
            aws_sdk_cloudwatchlogs::types::StartLiveTailResponseStream::SessionUpdate(
                live_tail_session_update,
            ) => {
                session_metadata = live_tail_session_update.session_metadata;
            }
            otherwise => panic!("received unexpected event type: {:?}", otherwise),
        }
    }

    replayer
        .validate(&["content-type", "content-length"], validate_success_body)
        .await
        .unwrap();

    assert_eq!(
        Some(LiveTailSessionMetadata::builder().sampled(false).build()),
        session_metadata
    );
}

async fn start_request(
    region: &'static str,
    events_json: &str,
) -> (ReplayingClient, StartLiveTailOutput) {
    let replayer = ReplayingClient::from_file(events_json).unwrap();

    let config = Config::builder()
        .region(Region::from_static(region))
        .http_client(replayer.clone())
        .with_test_defaults()
        .credentials_provider(Credentials::for_tests())
        .build();
    let client = Client::from_conf(config);

    let output = client
        .start_live_tail()
        .set_log_group_identifiers(Some(vec![format!("arn:aws:logs:{region}:123456789123:log-group:/aws/codebuild/CodeBuildS3PublisherProject-aBCdEfGHIjkO")]))
        .send()
        .await
        .unwrap();

    (replayer, output)
}

fn validate_success_body(
    expected_body: &[u8],
    actual_body: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    validate_body(expected_body, actual_body, true)
}
