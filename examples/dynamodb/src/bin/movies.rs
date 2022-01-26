/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::collections::HashMap;
use std::time::Duration;

use aws_config::meta::region::RegionProviderChain;
use aws_http::retry::AwsErrorRetryPolicy;
use aws_sdk_dynamodb::client::fluent_builders::Query;
use aws_sdk_dynamodb::error::DescribeTableError;
use aws_sdk_dynamodb::input::DescribeTableInput;
use aws_sdk_dynamodb::middleware::DefaultMiddleware;
use aws_sdk_dynamodb::model::{
    AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput,
    ScalarAttributeType, TableStatus,
};
use aws_sdk_dynamodb::operation::DescribeTable;
use aws_sdk_dynamodb::output::DescribeTableOutput;
use aws_sdk_dynamodb::{Client, Config, Error, Region, PKG_VERSION};
use aws_smithy_client::erase::DynConnector;
use aws_smithy_http::operation::Operation;
use aws_smithy_http::result::{SdkError, SdkSuccess};
use aws_smithy_http::retry::ClassifyResponse;
use aws_smithy_types::retry::RetryKind;
use serde_json::Value;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The name of the table.
    #[structopt(short, long)]
    table: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// A partial reimplementation of
/// <https://docs.amazonaws.cn/en_us/amazondynamodb/latest/developerguide/GettingStarted.Ruby.html>
/// in Rust
///
/// - Create table
/// - Wait for table to be ready
/// - Add a couple of rows
/// - Query for those rows
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        region,
        table,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;

    println!();

    if verbose {
        println!("DynamoDB client version: {}", PKG_VERSION);
        println!(
            "Region:                  {}",
            shared_config.region().unwrap()
        );
        println!("Table:                   {}", &table);
        println!();
    }

    let client = Client::new(&shared_config);

    let raw_client = aws_smithy_client::Client::<DynConnector, DefaultMiddleware>::dyn_https();

    let table_exists = does_table_exist(&client, &table).await?;

    if !table_exists {
        println!("Creating table.");

        create_table(&client, &table.to_string())
            .send()
            .await
            .expect("failed to create table");
    }

    raw_client
        .call(wait_for_ready_table(&table.to_string(), client.conf()).await)
        .await
        .expect("table should become ready");

    // data.json contains 2 movies from 2013
    let data = match serde_json::from_str(include_str!("data.json")).expect("should be valid JSON")
    {
        Value::Array(inner) => inner,
        data => panic!("data must be an array, got: {:?}", data),
    };

    println!("Adding items to table.");

    for value in data {
        add_item(&client, &table, value).await?;
    }

    println!("Making sure table has items.");

    let films_2222 = movies_in_year(&client, &table.to_string(), 2222)
        .send()
        .await
        .expect("query should succeed");

    // this isn't back to the future, there are no movies from 2022
    assert_eq!(films_2222.count(), 0);

    let films_2013 = movies_in_year(&client, &table.to_string(), 2013)
        .send()
        .await
        .expect("query should succeed");

    assert_eq!(films_2013.count(), 2);

    let titles: Vec<AttributeValue> = films_2013
        .items
        .unwrap()
        .iter_mut()
        .map(|row| row.remove("title").expect("row should have title"))
        .collect();

    assert_eq!(
        titles,
        vec![
            AttributeValue::S("Rush".to_string()),
            AttributeValue::S("Turn It Down, Or Else!".to_string())
        ]
    );

    println!("Deleting table.");

    delete_table(&client, &table).await
}

// Does table exist?
// snippet-start:[dynamodb.rust.movies-does_table_exist]
async fn does_table_exist(client: &Client, table: &str) -> Result<bool, Error> {
    let table_exists = client
        .list_tables()
        .send()
        .await
        .expect("should succeed")
        .table_names()
        .as_ref()
        .unwrap()
        .contains(&table.into());

    Ok(table_exists)
}
// snippet-end:[dynamodb.rust.movies-does_table_exist]

// snippet-start:[dynamodb.rust.movies-create_table]
fn create_table(
    client: &Client,
    table_name: &str,
) -> aws_sdk_dynamodb::client::fluent_builders::CreateTable {
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
}
// snippet-end:[dynamodb.rust.movies-create_table]

fn parse_item(value: Value) -> HashMap<String, AttributeValue> {
    match value_to_item(value) {
        AttributeValue::M(map) => map,
        other => panic!("can only insert top level values, got {:?}", other),
    }
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

// Add an item to the table.
// snippet-start:[dynamodb.rust.movies-add_item]
async fn add_item(client: &Client, table: &str, value: serde_json::Value) -> Result<(), Error> {
    client
        .put_item()
        .table_name(table)
        .set_item(Some(parse_item(value)))
        .send()
        .await?;

    Ok(())
}
// snippet-end:[dynamodb.rust.movies-add_item]

// snippet-start:[dynamodb.rust.movies-movies_in_year]
fn movies_in_year(client: &Client, table_name: &str, year: u16) -> Query {
    client
        .query()
        .table_name(table_name)
        .key_condition_expression("#yr = :yyyy")
        .expression_attribute_names("#yr", "year")
        .expression_attribute_values(":yyyy", AttributeValue::N(year.to_string()))
}
// snippet-end:[dynamodb.rust.movies-movies_in_year]

// Deletes a table.
// snippet-start:[dynamodb.rust.movies-delete_table]
async fn delete_table(client: &Client, table: &str) -> Result<(), Error> {
    client.delete_table().table_name(table).send().await?;

    Ok(())
}
// snippet-end:[dynamodb.rust.movies-delete_table]

/// Hand-written waiter to retry every second until the table is out of `Creating` state
#[derive(Clone)]
struct WaitForReadyTable<R> {
    inner: R,
}

impl<R> ClassifyResponse<SdkSuccess<DescribeTableOutput>, SdkError<DescribeTableError>>
    for WaitForReadyTable<R>
where
    R: ClassifyResponse<SdkSuccess<DescribeTableOutput>, SdkError<DescribeTableError>>,
{
    fn classify(
        &self,
        response: Result<&SdkSuccess<DescribeTableOutput>, &SdkError<DescribeTableError>>,
    ) -> RetryKind {
        match self.inner.classify(response) {
            RetryKind::NotRetryable => (),
            other => return other,
        };
        match response {
            Ok(SdkSuccess { parsed, .. }) => {
                if parsed
                    .table
                    .as_ref()
                    .unwrap()
                    .table_status
                    .as_ref()
                    .unwrap()
                    == &TableStatus::Creating
                {
                    RetryKind::Explicit(Duration::from_secs(1))
                } else {
                    RetryKind::NotRetryable
                }
            }
            _ => RetryKind::NotRetryable,
        }
    }
}

/// Construct a `DescribeTable` request with a policy to retry every second until the table
/// is ready
async fn wait_for_ready_table(
    table_name: &str,
    conf: &Config,
) -> Operation<DescribeTable, WaitForReadyTable<AwsErrorRetryPolicy>> {
    let operation = DescribeTableInput::builder()
        .table_name(table_name)
        .build()
        .expect("valid input")
        .make_operation(conf)
        .await
        .expect("valid operation");
    let waiting_policy = WaitForReadyTable {
        inner: operation.retry_policy().clone(),
    };
    operation.with_retry_policy(waiting_policy)
}
