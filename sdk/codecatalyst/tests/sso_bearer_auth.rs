/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_codecatalyst::config::Token;

#[tokio::test]
async fn sso_bearer_auth() {
    let replay = aws_smithy_http_client::test_util::dvr::ReplayingClient::from_file(
        "tests/sso_bearer_auth.json",
    )
    .unwrap();

    let config = aws_sdk_codecatalyst::Config::builder()
        .with_test_defaults()
        .http_client(replay.clone())
        .token_provider(Token::new("sso_bearer_auth_test", None))
        .build();
    let client = aws_sdk_codecatalyst::Client::from_conf(config);

    let response = client
        .list_spaces()
        .send()
        .await
        .expect("successful response");
    let item = &response.items.unwrap()[0];
    assert_eq!("somespacename", item.name);

    replay.relaxed_validate("application/json").await.unwrap();
}
