/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_rdsdata::{Client, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// The SQL query string.
    #[structopt(short, long)]
    query: String,

    /// The ARN of your Aurora serverless DB cluster.
    #[structopt(short, long)]
    resource_arn: String,

    /// The ARN of the Secrets Manager secret.
    #[structopt(short, long)]
    secret_arn: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Sends a query to an Aurora serverless cluster.
/// # Arguments
///
/// * `-q QUERY` - The SQL query to run against the cluster.
///    It should look something like: __"SELECT * FROM pg_catalog.pg_tables limit 1"__.
///    Don't forget you'll likely have to escape some characters.
/// * `-r RESOURCE_ARN` - The ARN of your Aurora serverless DB cluster.
///    It should look something like __arn:aws:rds:us-west-2:AWS_ACCOUNT:cluster:database-2__.
/// * `-s SECRET_ARN` - The ARN of the Secrets Manager secret.
///    It should look something like: __arn:aws:secretsmanager:us-west-2:AWS_ACCOUNT:secret:database2/test/postgres-b8maVb__.
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        default_region,
        query,
        resource_arn,
        secret_arn,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(default_region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    println!();

    if verbose {
        println!("RDS data version: {}", PKG_VERSION);
        println!("Region:           {:?}", shared_config.region().unwrap());
        println!("Resource ARN:     {}", &resource_arn);
        println!("Secrets ARN:      {}", &secret_arn);
        println!("Query:");
        println!("  {}", &query);
        println!();
    }

    let st = client
        .execute_statement()
        .resource_arn(resource_arn)
        .database("postgres") // Do not confuse this with db instance name
        .sql(query)
        .secret_arn(secret_arn);

    let result = st.send().await?;

    println!("{:?}", result);
    println!();

    Ok(())
}
