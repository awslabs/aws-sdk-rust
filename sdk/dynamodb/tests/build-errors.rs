/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::create_table::CreateTableError;
use aws_sdk_dynamodb::types::{KeySchemaElement, KeyType};
use aws_sdk_dynamodb::Client;

#[allow(dead_code)]
async fn create_table_test(client: &Client) -> Result<(), SdkError<CreateTableError>> {
    let _just_checking_compilation = client
        .create_table()
        .table_name("test")
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("year")
                .key_type(KeyType::Hash)
                .build()?,
        )
        .send()
        .await;
    Ok(())
}

#[allow(dead_code)]
async fn create_table_test_super_error(client: &Client) -> Result<(), aws_sdk_dynamodb::Error> {
    let _just_checking_compilation = client
        .create_table()
        .table_name("test")
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("year")
                .key_type(KeyType::Hash)
                .build()?,
        )
        .send()
        .await;
    Ok(())
}
