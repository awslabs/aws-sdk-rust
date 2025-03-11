/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_runtime::user_agent::test_util::assert_ua_contains_metric_values;
use aws_sdk_ec2::{client::Waiters, config::Region, error::DisplayErrorContext, Client};
use aws_smithy_async::test_util::tick_advance_sleep::{
    tick_advance_time_and_sleep, TickAdvanceTime,
};
use aws_smithy_http_client::test_util::dvr::ReplayingClient;
use aws_smithy_runtime::test_util::capture_test_logs::show_test_logs;
use aws_smithy_runtime_api::client::waiters::error::WaiterError;
use aws_smithy_types::retry::RetryConfig;
use std::time::Duration;

async fn prerequisites() -> (Client, ReplayingClient, TickAdvanceTime) {
    let (time_source, sleep_impl) = tick_advance_time_and_sleep();
    let client =
        ReplayingClient::from_file("tests/instance-status-ok-waiter-success.json").unwrap();
    let config = aws_sdk_ec2::Config::builder()
        .with_test_defaults()
        .http_client(client.clone())
        .time_source(time_source.clone())
        .sleep_impl(sleep_impl)
        .region(Region::new("us-west-2"))
        .retry_config(RetryConfig::standard())
        .build();
    (aws_sdk_ec2::Client::from_conf(config), client, time_source)
}

#[tokio::test]
async fn waiters_success() {
    let _logs = show_test_logs();

    let (ec2, http_client, time_source) = prerequisites().await;

    ec2.start_instances()
        .instance_ids("i-09fb4224219ac6902")
        .send()
        .await
        .unwrap();

    let waiter_task = tokio::spawn(
        ec2.wait_until_instance_status_ok()
            .instance_ids("i-09fb4224219ac6902")
            .wait(Duration::from_secs(300)),
    );

    // The responses in the test data will make the waiter poll a few times, so it will take some time
    // to complete. If successful, it shouldn't take a full 300 seconds. However, in the event it isn't successful,
    // waiting the full 300 seconds will result in a max time exceeded error instead of a never ending test.
    time_source.tick(Duration::from_secs(305)).await;
    waiter_task.await.unwrap().unwrap();

    http_client.full_validate("application/xml").await.unwrap();
}

#[tokio::test]
async fn waiters_exceed_max_wait_time() {
    let _logs = show_test_logs();

    let (ec2, _, time_source) = prerequisites().await;

    ec2.start_instances()
        .instance_ids("i-09fb4224219ac6902")
        .send()
        .await
        .unwrap();

    let waiter_task = tokio::spawn(
        ec2.wait_until_instance_status_ok()
            .instance_ids("i-09fb4224219ac6902")
            .wait(Duration::from_secs(30)),
    );

    time_source.tick(Duration::from_secs(35)).await;
    let err = waiter_task.await.unwrap().err().expect("should fail");
    match err {
        WaiterError::ExceededMaxWait(context) => {
            assert_eq!(30, context.max_wait().as_secs());
            assert_eq!(30, context.elapsed().as_secs());
            assert_eq!(3, context.poll_count());
        }
        err => panic!("unexpected error: {}", DisplayErrorContext(&err)),
    }
}

#[tokio::test]
async fn should_emit_business_metric_for_waiter_in_user_agent() {
    // This function has the same setup and execution as `waiters_success`, but differs in the verification step.
    // Because `full_validate` consumes the recorded requests after being called, we need a separate test
    // to examine these requests.

    let _logs = show_test_logs();

    let (ec2, http_client, time_source) = prerequisites().await;

    ec2.start_instances()
        .instance_ids("i-09fb4224219ac6902")
        .send()
        .await
        .unwrap();

    let waiter_task = tokio::spawn(
        ec2.wait_until_instance_status_ok()
            .instance_ids("i-09fb4224219ac6902")
            .wait(Duration::from_secs(300)),
    );

    time_source.tick(Duration::from_secs(305)).await;
    waiter_task.await.unwrap().unwrap();

    // Verify the corresponding business metric value has been emitted
    let actual_requests = http_client.take_requests().await;
    let user_agent_in_last_request = actual_requests
        .last()
        .unwrap()
        .headers()
        .get("x-amz-user-agent")
        .unwrap()
        .to_str()
        .unwrap();
    assert_ua_contains_metric_values(user_agent_in_last_request, &["B"]);
}
