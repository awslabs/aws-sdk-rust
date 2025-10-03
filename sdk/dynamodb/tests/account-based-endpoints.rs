/*
* Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
* SPDX-License-Identifier: Apache-2.0
*/

use aws_config::Region;
use aws_credential_types::Credentials;
use aws_runtime::user_agent::test_util::assert_ua_contains_metric_values;
use aws_sdk_dynamodb::{
    config::Builder,
    error::{DisplayErrorContext, SdkError},
    operation::batch_get_item::{BatchGetItemError, BatchGetItemOutput},
    types::{AttributeValue, KeysAndAttributes},
    Client, Config,
};
use aws_smithy_http_client::test_util::{capture_request, CaptureRequestReceiver};
use aws_smithy_runtime::assert_str_contains;
use aws_smithy_runtime_api::http::Response;
use aws_types::endpoint_config::AccountIdEndpointMode;

fn test_client(update_builder: fn(Builder) -> Builder) -> (Client, CaptureRequestReceiver) {
    let (http_client, request) = capture_request(None);
    let builder = Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(
            Credentials::builder()
                .account_id("123456789012")
                .access_key_id("ANOTREAL")
                .secret_access_key("notrealrnrELgWzOk3IfjzDKtFBhDby")
                .provider_name("test")
                .build(),
        )
        .http_client(http_client);
    (Client::from_conf(update_builder(builder).build()), request)
}

async fn call_operation(
    client: Client,
    table_name: &str,
) -> Result<BatchGetItemOutput, SdkError<BatchGetItemError, Response>> {
    let mut attr_v = std::collections::HashMap::new();
    attr_v.insert(":s".to_string(), AttributeValue::S("value".into()));
    let mut kv = std::collections::HashMap::new();
    kv.insert(":pk".to_string(), AttributeValue::M(attr_v));
    client
        .batch_get_item()
        .request_items(
            table_name,
            KeysAndAttributes::builder().keys(kv).build().unwrap(),
        )
        .send()
        .await
}

#[tokio::test]
async fn basic_positive_cases() {
    let test_cases: &[(fn(Builder) -> Builder, &str, &str, &[&'static str])] = &[
        (
            std::convert::identity,
            "arn:aws:dynamodb:us-east-1:333333333333:table/table_name",
            "https://333333333333.ddb.us-east-1.amazonaws.com/",
            &["P", "T"],
        ),
        (
            std::convert::identity,
            "table_name", // doesn't specify ARN for the table name
            "https://123456789012.ddb.us-east-1.amazonaws.com/", // the account ID should come from credentials
            &["P", "T"],
        ),
        (
            |b: Builder| b.credentials_provider(Credentials::for_tests()), // credentials do not provide an account ID
            "arn:aws:dynamodb:us-east-1:333333333333:table/table_name",
            "https://333333333333.ddb.us-east-1.amazonaws.com/",
            &["P"],
        ),
        (
            |b: Builder| b.account_id_endpoint_mode(AccountIdEndpointMode::Preferred), // sets the default mode `Preferred` explicitly
            "arn:aws:dynamodb:us-east-1:333333333333:table/table_name",
            "https://333333333333.ddb.us-east-1.amazonaws.com/",
            &["P", "T"],
        ),
        (
            |b: Builder| b.account_id_endpoint_mode(AccountIdEndpointMode::Disabled),
            "arn:aws:dynamodb:us-east-1:333333333333:table/table_name",
            "https://dynamodb.us-east-1.amazonaws.com/",
            &["Q", "T"],
        ),
        (
            |b: Builder| b.account_id_endpoint_mode(AccountIdEndpointMode::Required),
            "arn:aws:dynamodb:us-east-1:333333333333:table/table_name",
            "https://333333333333.ddb.us-east-1.amazonaws.com/",
            &["R", "T"],
        ),
    ];

    for (i, (update_builder, table_name, expected_uri, expected_metrics)) in
        test_cases.into_iter().enumerate()
    {
        let (client, rx) = test_client(*update_builder);
        let _ = call_operation(client, table_name).await;
        let req = rx.expect_request();
        assert_eq!(
            *expected_uri,
            req.uri(),
            "on the {i}th test case where table name is `{table_name}`"
        );

        // Test User-Agent metrics for account ID tracking
        let user_agent = req.headers().get("x-amz-user-agent").unwrap();
        assert_ua_contains_metric_values(user_agent, expected_metrics);
    }
}

#[tokio::test]
async fn error_should_be_raised_when_account_id_is_expected_but_not_resolved() {
    let (client, _) = test_client(|b| {
        b.account_id_endpoint_mode(AccountIdEndpointMode::Required)
            .credentials_provider(Credentials::for_tests())
    });
    // doesn't specify ARN for the table name
    let err = call_operation(client, "table_name")
        .await
        .err()
        .expect("request should fail");

    assert_str_contains!(
        format!("{}", DisplayErrorContext(err)),
        "AccountIdEndpointMode is required but no AccountID was provided or able to be loaded"
    );
}
