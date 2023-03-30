/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb as dynamodb;
use aws_smithy_client::test_connection::TestConnection;
use aws_smithy_http::body::SdkBody;
use dynamodb::config::{Credentials, Region};
use dynamodb::operation::query::QueryOutput;
use dynamodb::types::{
    AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput,
    ScalarAttributeType, TableStatus,
};
use dynamodb::Client;
use http::header::{HeaderName, AUTHORIZATION};
use http::Uri;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::Instant;

async fn create_table(client: &Client, table_name: &str) {
    client
        .create_table()
        .table_name(table_name)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("year")
                .key_type(KeyType::Hash)
                .build(),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("title")
                .key_type(KeyType::Range)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("year")
                .attribute_type(ScalarAttributeType::N)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("title")
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(10)
                .write_capacity_units(10)
                .build(),
        )
        .send()
        .await
        .expect("failed to create table");
}

fn value_to_item(value: Value) -> AttributeValue {
    match value {
        Value::Null => AttributeValue::Null(true),
        Value::Bool(b) => AttributeValue::Bool(b),
        Value::Number(n) => AttributeValue::N(n.to_string()),
        Value::String(s) => AttributeValue::S(s),
        Value::Array(a) => AttributeValue::L(a.into_iter().map(value_to_item).collect()),
        Value::Object(o) => {
            AttributeValue::M(o.into_iter().map(|(k, v)| (k, value_to_item(v))).collect())
        }
    }
}

async fn add_item(client: &Client, table_name: impl Into<String>, item: Value) {
    let attribute_value = match value_to_item(item) {
        AttributeValue::M(map) => map,
        other => panic!("can only insert top level values, got {:?}", other),
    };

    client
        .put_item()
        .table_name(table_name)
        .set_item(Some(attribute_value))
        .send()
        .await
        .expect("valid operation");
}

async fn movies_in_year(client: &Client, table_name: &str, year: u16) -> QueryOutput {
    let mut expr_attrib_names = HashMap::new();
    expr_attrib_names.insert("#yr".to_string(), "year".to_string());
    let mut expr_attrib_values = HashMap::new();
    expr_attrib_values.insert(":yyyy".to_string(), AttributeValue::N(year.to_string()));

    client
        .query()
        .table_name(table_name)
        .key_condition_expression("#yr = :yyyy")
        .set_expression_attribute_names(Some(expr_attrib_names))
        .set_expression_attribute_values(Some(expr_attrib_values))
        .send()
        .await
        .expect("valid operation")
}

/// Poll the DescribeTable operation once per second until the table exists.
async fn wait_for_ready_table(client: &Client, table_name: &str) {
    loop {
        if let Some(table) = client
            .describe_table()
            .table_name(table_name)
            .send()
            .await
            .expect("success")
            .table()
        {
            if !matches!(table.table_status, Some(TableStatus::Creating)) {
                break;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Validate that time has passed with a 5ms tolerance
///
/// This is to account for some non-determinism in the Tokio timer
fn assert_time_passed(initial: Instant, passed: Duration) {
    let now = tokio::time::Instant::now();
    let delta = now - initial;
    if (delta.as_millis() as i128 - passed.as_millis() as i128).abs() > 5 {
        assert_eq!(delta, passed)
    }
}

/// A partial reimplementation of https://docs.amazonaws.cn/en_us/amazondynamodb/latest/developerguide/GettingStarted.Ruby.html
/// in Rust
///
/// - Create table
/// - Wait for table to be ready
/// - Add a couple of rows
/// - Query for those rows
#[tokio::test]
async fn movies_it() {
    let table_name = "Movies-5";
    // The waiter will retry 5 times
    tokio::time::pause();
    let conn = movies_it_test_connection(); // RecordingConnection::https();
    let conf = dynamodb::Config::builder()
        .region(Region::new("us-east-1"))
        .http_connector(conn.clone())
        .credentials_provider(Credentials::for_tests())
        .build();
    let client = Client::from_conf(conf);

    create_table(&client, table_name).await;

    let waiter_start = tokio::time::Instant::now();
    wait_for_ready_table(&client, table_name).await;

    assert_time_passed(waiter_start, Duration::from_secs(4));
    // data.json contains 2 movies from 2013
    let data = match serde_json::from_str(include_str!("data.json")).expect("should be valid JSON")
    {
        Value::Array(inner) => inner,
        data => panic!("data must be an array, got: {:?}", data),
    };
    for item in data {
        add_item(&client, table_name, item.clone()).await;
    }
    let films_2222 = movies_in_year(&client, table_name, 2222).await;
    // this isn't "Back To The Future", there are no movies from 2222
    assert_eq!(films_2222.count, 0);

    let films_2013 = movies_in_year(&client, table_name, 2013).await;
    assert_eq!(films_2013.count, 2);
    let titles: Vec<AttributeValue> = films_2013
        .items
        .unwrap()
        .into_iter()
        .map(|mut row| row.remove("title").expect("row should have title"))
        .collect();
    assert_eq!(
        titles,
        vec![
            AttributeValue::S("Rush".to_string()),
            AttributeValue::S("Turn It Down, Or Else!".to_string())
        ]
    );

    conn.assert_requests_match(&[AUTHORIZATION, HeaderName::from_static("x-amz-date")]);
}

/// Test connection for the movies IT
/// headers are signed with actual creds, at some point we could replace them with verifiable test
/// credentials, but there are plenty of other tests that target signing
fn movies_it_test_connection() -> TestConnection<&'static str> {
    TestConnection::new(vec![(
                                 http::Request::builder()
                                     .header("content-type", "application/x-amz-json-1.0")
                                     .header("x-amz-target", "DynamoDB_20120810.CreateTable")
                                     .header("content-length", "313")
                                     .header("authorization", "AWS4-HMAC-SHA256 Credential=ASIAR6OFQKMAFQIIYZ5T/20210308/us-east-1/dynamodb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=4a832eba37651836b524b587986be607607b077ad133c57b4bf7300d2e02f476")
                                     .header("x-amz-date", "20210308T155118Z")
                                     .uri(Uri::from_static("https://dynamodb.us-east-1.amazonaws.com/"))
                                     .body(SdkBody::from(r#"{"AttributeDefinitions":[{"AttributeName":"year","AttributeType":"N"},{"AttributeName":"title","AttributeType":"S"}],"TableName":"Movies-5","KeySchema":[{"AttributeName":"year","KeyType":"HASH"},{"AttributeName":"title","KeyType":"RANGE"}],"ProvisionedThroughput":{"ReadCapacityUnits":10,"WriteCapacityUnits":10}}"#)).unwrap(),
                                 http::Response::builder()
                                     .header("server", "Server")
                                     .header("date", "Mon, 08 Mar 2021 15:51:18 GMT")
                                     .header("content-type", "application/x-amz-json-1.0")
                                     .header("content-length", "572")
                                     .header("connection", "keep-alive")
                                     .header("x-amzn-requestid", "RCII0AALE00UALC7LJ9AD600B7VV4KQNSO5AEMVJF66Q9ASUAAJG")
                                     .header("x-amz-crc32", "3715137447")
                                     .status(http::StatusCode::from_u16(200).unwrap())
                                     .body(r#"{"TableDescription":{"AttributeDefinitions":[{"AttributeName":"title","AttributeType":"S"},{"AttributeName":"year","AttributeType":"N"}],"CreationDateTime":1.615218678973E9,"ItemCount":0,"KeySchema":[{"AttributeName":"year","KeyType":"HASH"},{"AttributeName":"title","KeyType":"RANGE"}],"ProvisionedThroughput":{"NumberOfDecreasesToday":0,"ReadCapacityUnits":10,"WriteCapacityUnits":10},"TableArn":"arn:aws:dynamodb:us-east-1:134095065856:table/Movies-5","TableId":"b08c406a-7dbc-4f7d-b7c6-672a43ec21cd","TableName":"Movies-5","TableSizeBytes":0,"TableStatus":"CREATING"}}"#).unwrap()),
                             (http::Request::builder()
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("x-amz-target", "DynamoDB_20120810.DescribeTable")
                                  .header("content-length", "24")
                                  .header("authorization", "AWS4-HMAC-SHA256 Credential=ASIAR6OFQKMAFQIIYZ5T/20210308/us-east-1/dynamodb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=01b0129a2a4fb3af14559fde8163d59de9c43907152a12479002b3a7c75fa0df")
                                  .header("x-amz-date", "20210308T155119Z")
                                  .uri(Uri::from_static("https://dynamodb.us-east-1.amazonaws.com/"))
                                  .body(SdkBody::from(r#"{"TableName":"Movies-5"}"#)).unwrap(),
                              http::Response::builder()
                                  .header("server", "Server")
                                  .header("date", "Mon, 08 Mar 2021 15:51:18 GMT")
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("content-length", "561")
                                  .header("connection", "keep-alive")
                                  .header("x-amzn-requestid", "O1C6QKCG8GT7D2K922T4QRL9N3VV4KQNSO5AEMVJF66Q9ASUAAJG")
                                  .header("x-amz-crc32", "46742265")
                                  .status(http::StatusCode::from_u16(200).unwrap())
                                  .body(r#"{"Table":{"AttributeDefinitions":[{"AttributeName":"title","AttributeType":"S"},{"AttributeName":"year","AttributeType":"N"}],"CreationDateTime":1.615218678973E9,"ItemCount":0,"KeySchema":[{"AttributeName":"year","KeyType":"HASH"},{"AttributeName":"title","KeyType":"RANGE"}],"ProvisionedThroughput":{"NumberOfDecreasesToday":0,"ReadCapacityUnits":10,"WriteCapacityUnits":10},"TableArn":"arn:aws:dynamodb:us-east-1:134095065856:table/Movies-5","TableId":"b08c406a-7dbc-4f7d-b7c6-672a43ec21cd","TableName":"Movies-5","TableSizeBytes":0,"TableStatus":"CREATING"}}"#).unwrap()),
                             (http::Request::builder()
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("x-amz-target", "DynamoDB_20120810.DescribeTable")
                                  .header("content-length", "24")
                                  .header("authorization", "AWS4-HMAC-SHA256 Credential=ASIAR6OFQKMAFQIIYZ5T/20210308/us-east-1/dynamodb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=7f3a743bb460f26296640ae775d282f0153eda750855ec00ace1815becfd2de5")
                                  .header("x-amz-date", "20210308T155120Z")
                                  .uri(Uri::from_static("https://dynamodb.us-east-1.amazonaws.com/")).body(SdkBody::from(r#"{"TableName":"Movies-5"}"#)).unwrap(),
                              http::Response::builder()
                                  .header("server", "Server")
                                  .header("date", "Mon, 08 Mar 2021 15:51:20 GMT")
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("content-length", "561")
                                  .header("connection", "keep-alive")
                                  .header("x-amzn-requestid", "EN5N26BO1FAOEMUUSD7B7SUPPVVV4KQNSO5AEMVJF66Q9ASUAAJG")
                                  .header("x-amz-crc32", "46742265")
                                  .status(http::StatusCode::from_u16(200).unwrap())
                                  .body(r#"{"Table":{"AttributeDefinitions":[{"AttributeName":"title","AttributeType":"S"},{"AttributeName":"year","AttributeType":"N"}],"CreationDateTime":1.615218678973E9,"ItemCount":0,"KeySchema":[{"AttributeName":"year","KeyType":"HASH"},{"AttributeName":"title","KeyType":"RANGE"}],"ProvisionedThroughput":{"NumberOfDecreasesToday":0,"ReadCapacityUnits":10,"WriteCapacityUnits":10},"TableArn":"arn:aws:dynamodb:us-east-1:134095065856:table/Movies-5","TableId":"b08c406a-7dbc-4f7d-b7c6-672a43ec21cd","TableName":"Movies-5","TableSizeBytes":0,"TableStatus":"CREATING"}}"#).unwrap()),
                             (http::Request::builder()
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("x-amz-target", "DynamoDB_20120810.DescribeTable")
                                  .header("content-length", "24")
                                  .header("authorization", "AWS4-HMAC-SHA256 Credential=ASIAR6OFQKMAFQIIYZ5T/20210308/us-east-1/dynamodb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=46a148c560139bc0da171bd915ea8c0b96a7012629f5db7b6bf70fcd1a66fd24")
                                  .header("x-amz-date", "20210308T155121Z")
                                  .uri(Uri::from_static("https://dynamodb.us-east-1.amazonaws.com/"))
                                  .body(SdkBody::from(r#"{"TableName":"Movies-5"}"#)).unwrap(),
                              http::Response::builder()
                                  .header("server", "Server")
                                  .header("date", "Mon, 08 Mar 2021 15:51:21 GMT")
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("content-length", "561")
                                  .header("connection", "keep-alive")
                                  .header("x-amzn-requestid", "PHCMGEVI6JLN9JNMKSSA3M76H3VV4KQNSO5AEMVJF66Q9ASUAAJG")
                                  .header("x-amz-crc32", "46742265")
                                  .status(http::StatusCode::from_u16(200).unwrap())
                                  .body(r#"{"Table":{"AttributeDefinitions":[{"AttributeName":"title","AttributeType":"S"},{"AttributeName":"year","AttributeType":"N"}],"CreationDateTime":1.615218678973E9,"ItemCount":0,"KeySchema":[{"AttributeName":"year","KeyType":"HASH"},{"AttributeName":"title","KeyType":"RANGE"}],"ProvisionedThroughput":{"NumberOfDecreasesToday":0,"ReadCapacityUnits":10,"WriteCapacityUnits":10},"TableArn":"arn:aws:dynamodb:us-east-1:134095065856:table/Movies-5","TableId":"b08c406a-7dbc-4f7d-b7c6-672a43ec21cd","TableName":"Movies-5","TableSizeBytes":0,"TableStatus":"CREATING"}}"#).unwrap()),
                             (http::Request::builder()
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("x-amz-target", "DynamoDB_20120810.DescribeTable")
                                  .header("content-length", "24")
                                  .header("authorization", "AWS4-HMAC-SHA256 Credential=ASIAR6OFQKMAFQIIYZ5T/20210308/us-east-1/dynamodb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=15bb7c9b2350747d62349091b3ea59d9e1800d1dca04029943329259bba85cb4")
                                  .header("x-amz-date", "20210308T155122Z")
                                  .uri(Uri::from_static("https://dynamodb.us-east-1.amazonaws.com/"))
                                  .body(SdkBody::from(r#"{"TableName":"Movies-5"}"#)).unwrap(),
                              http::Response::builder()
                                  .header("server", "Server")
                                  .header("date", "Mon, 08 Mar 2021 15:51:22 GMT")
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("content-length", "561")
                                  .header("connection", "keep-alive")
                                  .header("x-amzn-requestid", "1Q22O983HD3511TN6Q5RRTP0MFVV4KQNSO5AEMVJF66Q9ASUAAJG")
                                  .header("x-amz-crc32", "46742265")
                                  .status(http::StatusCode::from_u16(200).unwrap())
                                  .body(r#"{"Table":{"AttributeDefinitions":[{"AttributeName":"title","AttributeType":"S"},{"AttributeName":"year","AttributeType":"N"}],"CreationDateTime":1.615218678973E9,"ItemCount":0,"KeySchema":[{"AttributeName":"year","KeyType":"HASH"},{"AttributeName":"title","KeyType":"RANGE"}],"ProvisionedThroughput":{"NumberOfDecreasesToday":0,"ReadCapacityUnits":10,"WriteCapacityUnits":10},"TableArn":"arn:aws:dynamodb:us-east-1:134095065856:table/Movies-5","TableId":"b08c406a-7dbc-4f7d-b7c6-672a43ec21cd","TableName":"Movies-5","TableSizeBytes":0,"TableStatus":"CREATING"}}"#).unwrap()),
                             (http::Request::builder()
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("x-amz-target", "DynamoDB_20120810.DescribeTable")
                                  .header("content-length", "24")
                                  .header("authorization", "AWS4-HMAC-SHA256 Credential=ASIAR6OFQKMAFQIIYZ5T/20210308/us-east-1/dynamodb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=6d0a78087bc112c68a91b4b2d457efd8c09149b85b8f998f8c4b3f9916c8a743")
                                  .header("x-amz-date", "20210308T155123Z")
                                  .uri(Uri::from_static("https://dynamodb.us-east-1.amazonaws.com/"))
                                  .body(SdkBody::from(r#"{"TableName":"Movies-5"}"#)).unwrap(),
                              http::Response::builder()
                                  .header("server", "Server")
                                  .header("date", "Mon, 08 Mar 2021 15:51:23 GMT")
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("content-length", "559")
                                  .header("connection", "keep-alive")
                                  .header("x-amzn-requestid", "ONJBNV2A9GBNUT34KH73JLL23BVV4KQNSO5AEMVJF66Q9ASUAAJG")
                                  .header("x-amz-crc32", "24113616")
                                  .status(http::StatusCode::from_u16(200).unwrap())
                                  .body(r#"{"Table":{"AttributeDefinitions":[{"AttributeName":"title","AttributeType":"S"},{"AttributeName":"year","AttributeType":"N"}],"CreationDateTime":1.615218678973E9,"ItemCount":0,"KeySchema":[{"AttributeName":"year","KeyType":"HASH"},{"AttributeName":"title","KeyType":"RANGE"}],"ProvisionedThroughput":{"NumberOfDecreasesToday":0,"ReadCapacityUnits":10,"WriteCapacityUnits":10},"TableArn":"arn:aws:dynamodb:us-east-1:134095065856:table/Movies-5","TableId":"b08c406a-7dbc-4f7d-b7c6-672a43ec21cd","TableName":"Movies-5","TableSizeBytes":0,"TableStatus":"ACTIVE"}}"#).unwrap()),
                             (http::Request::builder()
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("x-amz-target", "DynamoDB_20120810.PutItem")
                                  .header("content-length", "619")
                                  .header("authorization", "AWS4-HMAC-SHA256 Credential=ASIAR6OFQKMAFQIIYZ5T/20210308/us-east-1/dynamodb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=85fc7d2064a0e6d9c38d64751d39d311ad415ae4079ef21ef254b23ecf093519")
                                  .header("x-amz-date", "20210308T155123Z")
                                  .uri(Uri::from_static("https://dynamodb.us-east-1.amazonaws.com/"))
                                  .body(SdkBody::from(r#"{"TableName":"Movies-5","Item":{"info":{"M":{"rating":{"N":"6.2"},"genres":{"L":[{"S":"Comedy"},{"S":"Drama"}]},"image_url":{"S":"http://ia.media-imdb.com/images/N/O9ERWAU7FS797AJ7LU8HN09AMUP908RLlo5JF90EWR7LJKQ7@@._V1_SX400_.jpg"},"release_date":{"S":"2013-01-18T00:00:00Z"},"actors":{"L":[{"S":"David Matthewman"},{"S":"Ann Thomas"},{"S":"Jonathan G. Neff"}]},"plot":{"S":"A rock band plays their music at high volumes, annoying the neighbors."},"running_time_secs":{"N":"5215"},"rank":{"N":"11"},"directors":{"L":[{"S":"Alice Smith"},{"S":"Bob Jones"}]}}},"title":{"S":"Turn It Down, Or Else!"},"year":{"N":"2013"}}}"#)).unwrap(),
                              http::Response::builder()
                                  .header("server", "Server")
                                  .header("date", "Mon, 08 Mar 2021 15:51:23 GMT")
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("content-length", "2")
                                  .header("connection", "keep-alive")
                                  .header("x-amzn-requestid", "E6TGS5HKHHV08HSQA31IO1IDMFVV4KQNSO5AEMVJF66Q9ASUAAJG")
                                  .header("x-amz-crc32", "2745614147")
                                  .status(http::StatusCode::from_u16(200).unwrap())
                                  .body(r#"{}"#).unwrap()),
                             (http::Request::builder()
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("x-amz-target", "DynamoDB_20120810.PutItem")
                                  .header("content-length", "636")
                                  .header("authorization", "AWS4-HMAC-SHA256 Credential=ASIAR6OFQKMAFQIIYZ5T/20210308/us-east-1/dynamodb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=e4b1658c9f5129b3656381f6592a30e0061b1566263fbf27d982817ea79483f6")
                                  .header("x-amz-date", "20210308T155123Z")
                                  .uri(Uri::from_static("https://dynamodb.us-east-1.amazonaws.com/"))
                                  .body(SdkBody::from(r#"{"TableName":"Movies-5","Item":{"info":{"M":{"plot":{"S":"A re-creation of the merciless 1970s rivalry between Formula One rivals James Hunt and Niki Lauda."},"rating":{"N":"8.3"},"rank":{"N":"2"},"release_date":{"S":"2013-09-02T00:00:00Z"},"directors":{"L":[{"S":"Ron Howard"}]},"image_url":{"S":"http://ia.media-imdb.com/images/M/MV5BMTQyMDE0MTY0OV5BMl5BanBnXkFtZTcwMjI2OTI0OQ@@._V1_SX400_.jpg"},"actors":{"L":[{"S":"Daniel Bruhl"},{"S":"Chris Hemsworth"},{"S":"Olivia Wilde"}]},"running_time_secs":{"N":"7380"},"genres":{"L":[{"S":"Action"},{"S":"Biography"},{"S":"Drama"},{"S":"Sport"}]}}},"title":{"S":"Rush"},"year":{"N":"2013"}}}"#)).unwrap(),
                              http::Response::builder()
                                  .header("server", "Server")
                                  .header("date", "Mon, 08 Mar 2021 15:51:23 GMT")
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("content-length", "2")
                                  .header("connection", "keep-alive")
                                  .header("x-amzn-requestid", "B63D54LP2FOGQK9JE5KLJT49HJVV4KQNSO5AEMVJF66Q9ASUAAJG")
                                  .header("x-amz-crc32", "2745614147")
                                  .status(http::StatusCode::from_u16(200).unwrap())
                                  .body(r#"{}"#).unwrap()),
                             (http::Request::builder()
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("x-amz-target", "DynamoDB_20120810.Query")
                                  .header("content-length", "156")
                                  .header("authorization", "AWS4-HMAC-SHA256 Credential=ASIAR6OFQKMAFQIIYZ5T/20210308/us-east-1/dynamodb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=c9a0fdd0c7c3a792faddabca1fc154c8fbb54ddee7b06a8082e1c587615198b5")
                                  .header("x-amz-date", "20210308T155123Z")
                                  .uri(Uri::from_static("https://dynamodb.us-east-1.amazonaws.com/"))
                                  .body(SdkBody::from(r##"{"TableName":"Movies-5","KeyConditionExpression":"#yr = :yyyy","ExpressionAttributeNames":{"#yr":"year"},"ExpressionAttributeValues":{":yyyy":{"N":"2222"}}}"##)).unwrap(),
                              http::Response::builder()
                                  .header("server", "Server")
                                  .header("date", "Mon, 08 Mar 2021 15:51:23 GMT")
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("content-length", "39")
                                  .header("connection", "keep-alive")
                                  .header("x-amzn-requestid", "AUAS9KJ0TK9BSR986TRPC2RGTRVV4KQNSO5AEMVJF66Q9ASUAAJG")
                                  .header("x-amz-crc32", "3413411624")
                                  .status(http::StatusCode::from_u16(200).unwrap())
                                  .body(r#"{"Count":0,"Items":[],"ScannedCount":0}"#).unwrap()),
                             (http::Request::builder()
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("x-amz-target", "DynamoDB_20120810.Query")
                                  .header("content-length", "156")
                                  .header("authorization", "AWS4-HMAC-SHA256 Credential=ASIAR6OFQKMAFQIIYZ5T/20210308/us-east-1/dynamodb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=504d6b4de7093b20255b55057085937ec515f62f3c61da68c03bff3f0ce8a160")
                                  .header("x-amz-date", "20210308T155123Z")
                                  .uri(Uri::from_static("https://dynamodb.us-east-1.amazonaws.com/"))
                                  .body(SdkBody::from(r##"{"TableName":"Movies-5","KeyConditionExpression":"#yr = :yyyy","ExpressionAttributeNames":{"#yr":"year"},"ExpressionAttributeValues":{":yyyy":{"N":"2013"}}}"##)).unwrap(),
                              http::Response::builder()
                                  .header("server", "Server")
                                  .header("date", "Mon, 08 Mar 2021 15:51:23 GMT")
                                  .header("content-type", "application/x-amz-json-1.0")
                                  .header("content-length", "1231")
                                  .header("connection", "keep-alive")
                                  .header("x-amzn-requestid", "A5FGSJ9ET4OKB8183S9M47RQQBVV4KQNSO5AEMVJF66Q9ASUAAJG")
                                  .header("x-amz-crc32", "624725176")
                                  .status(http::StatusCode::from_u16(200).unwrap())
                                  .body(r#"{"Count":2,"Items":[{"year":{"N":"2013"},"info":{"M":{"actors":{"L":[{"S":"Daniel Bruhl"},{"S":"Chris Hemsworth"},{"S":"Olivia Wilde"}]},"plot":{"S":"A re-creation of the merciless 1970s rivalry between Formula One rivals James Hunt and Niki Lauda."},"release_date":{"S":"2013-09-02T00:00:00Z"},"image_url":{"S":"http://ia.media-imdb.com/images/M/MV5BMTQyMDE0MTY0OV5BMl5BanBnXkFtZTcwMjI2OTI0OQ@@._V1_SX400_.jpg"},"genres":{"L":[{"S":"Action"},{"S":"Biography"},{"S":"Drama"},{"S":"Sport"}]},"directors":{"L":[{"S":"Ron Howard"}]},"rating":{"N":"8.3"},"rank":{"N":"2"},"running_time_secs":{"N":"7380"}}},"title":{"S":"Rush"}},{"year":{"N":"2013"},"info":{"M":{"actors":{"L":[{"S":"David Matthewman"},{"S":"Ann Thomas"},{"S":"Jonathan G. Neff"}]},"release_date":{"S":"2013-01-18T00:00:00Z"},"plot":{"S":"A rock band plays their music at high volumes, annoying the neighbors."},"genres":{"L":[{"S":"Comedy"},{"S":"Drama"}]},"image_url":{"S":"http://ia.media-imdb.com/images/N/O9ERWAU7FS797AJ7LU8HN09AMUP908RLlo5JF90EWR7LJKQ7@@._V1_SX400_.jpg"},"directors":{"L":[{"S":"Alice Smith"},{"S":"Bob Jones"}]},"rating":{"N":"6.2"},"rank":{"N":"11"},"running_time_secs":{"N":"5215"}}},"title":{"S":"Turn It Down, Or Else!"}}],"ScannedCount":2}"#).unwrap())
    ])
}
