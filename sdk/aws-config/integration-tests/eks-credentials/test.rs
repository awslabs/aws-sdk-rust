/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::model::AttributeValue;
#[tokio::main]
async fn main() {
    let conf = aws_config::load_from_env().await;
    let dynamo = aws_sdk_dynamodb::Client::new(&conf);
    println!(
        "{:?}",
        dynamo
            .get_item()
            .key("id", AttributeValue::S("foo".into()))
            .table_name("EksCredentialsStack-TableCD117FA1-18ZPICQWXOPW")
            .send()
            .await
    );
}
