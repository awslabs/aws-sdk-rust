/*
* Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
* SPDX-License-Identifier: Apache-2.0
*/

use aws_config::Region;
use aws_credential_types::Credentials;
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

fn test_client<F>(update_builder: F) -> (Client, CaptureRequestReceiver)
where
    F: Fn(Builder) -> Builder,
{
    let (http_client, request) = capture_request(None);
    let builder = Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(
            Credentials::builder()
                .account_id("333333333333")
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
) -> Result<BatchGetItemOutput, SdkError<BatchGetItemError, Response>> {
    let mut attr_v = std::collections::HashMap::new();
    attr_v.insert(":s".to_string(), AttributeValue::S("value".into()));
    let mut kv = std::collections::HashMap::new();
    kv.insert(":pk".to_string(), AttributeValue::M(attr_v));
    client
        .batch_get_item()
        .request_items(
            "arn:aws:dynamodb:us-east-1:333333333333:table/table_name",
            KeysAndAttributes::builder().keys(kv).build().unwrap(),
        )
        .send()
        .await
}

#[tokio::test]
async fn account_id_should_be_included_in_request_uri() {
    // With the default `AccountIdEndpointMode::Preferred`
    {
        let (client, rx) = test_client(std::convert::identity);
        let _ = call_operation(client).await;
        let req = rx.expect_request();
        assert_eq!(
            "https://333333333333.ddb.us-east-1.amazonaws.com/",
            req.uri()
        )
    }

    // With `AccountIdEndpointMode::Required`
    {
        let (client, rx) =
            test_client(|b| b.account_id_endpoint_mode(AccountIdEndpointMode::Required));
        let _ = call_operation(client).await;
        let req = rx.expect_request();
        assert_eq!(
            "https://333333333333.ddb.us-east-1.amazonaws.com/",
            req.uri()
        )
    }
}

#[tokio::test]
async fn account_id_should_not_be_included_in_request_uri() {
    // If we disable the account-based endpoints, the resulting URI should not include the account ID.
    {
        let (client, rx) =
            test_client(|b| b.account_id_endpoint_mode(AccountIdEndpointMode::Disabled));
        let _ = call_operation(client).await;
        let req = rx.expect_request();
        assert_eq!("https://dynamodb.us-east-1.amazonaws.com/", req.uri());
    }

    // If credentials do not include the account ID, neither should the resulting URI.
    {
        let (client, rx) = test_client(|b| b.credentials_provider(Credentials::for_tests()));
        let _ = call_operation(client).await;
        let req = rx.expect_request();
        assert_eq!("https://dynamodb.us-east-1.amazonaws.com/", req.uri());
    }
}

#[tokio::test]
async fn error_should_be_raised_when_account_id_is_expected_but_not_provided() {
    let (client, _) = test_client(|b| {
        b.account_id_endpoint_mode(AccountIdEndpointMode::Required)
            .credentials_provider(Credentials::for_tests())
    });
    let err = call_operation(client)
        .await
        .err()
        .expect("request should fail");

    assert_str_contains!(
        format!("{}", DisplayErrorContext(err)),
        "AccountIdEndpointMode is required but no AccountID was provided or able to be loaded"
    );
}
