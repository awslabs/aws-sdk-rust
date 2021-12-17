/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_apigatewaymanagement::{config, Blob, Client, Endpoint, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// AWS apigatewaymanagenent must be used with a custom endpoint, which this example demonstrates how to set.
///
/// Usage:
/// 1. Setup a Websocket API Gateway endpoint with a route configured.
/// 2. Connect to the route with `wscat`: `wscat -c wss://<api-id>.execute-api.<region>.amazonaws.com/<stage>/`
/// 2. Determine the connection id (eg. by configuring your route to echo the connection id into the websocket)
/// 3. Invoke this example. The `data` sent should appear in `wscat`
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,

    /// API ID for your API
    #[structopt(short, long)]
    api_id: String,

    /// Deployment stage for your API
    #[structopt(short, long)]
    stage: String,

    /// Connection Id to send data to
    #[structopt(short, long)]
    connection_id: String,

    /// Data to send to the connection
    #[structopt(short, long)]
    data: String,
}

/// Displays information about the Amazon API Gateway REST APIs in the Region.
///
/// # Arguments
///
/// * `--api-id` - API ID for your API
/// * `--stage` - Stage for your API
/// * `[-r REGION]` - The Region in which the client is created.
///   If not supplied, uses the value of the **AWS_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let Opt {
        region,
        verbose,
        api_id,
        stage,
        connection_id,
        data,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    println!();

    let region = region_provider.region().await.expect("region must be set");
    if verbose {
        println!("APIGatewayManagement client version: {}", PKG_VERSION);
        println!("Region:                    {}", region.as_ref());

        println!();
    }

    let uri = format!(
        "https://{api_id}.execute-api.{region}.amazonaws.com/{stage}",
        api_id = api_id,
        region = region,
        stage = stage
    )
    .parse()
    .expect("could not construct valid URI for endpoint");
    let endpoint = Endpoint::immutable(uri);

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let api_management_config = config::Builder::from(&shared_config)
        .endpoint_resolver(endpoint)
        .build();
    let client = Client::from_conf(api_management_config);

    client
        .post_to_connection()
        .connection_id(connection_id)
        .data(Blob::new(data))
        .send()
        .await?;
    Ok(())
}
