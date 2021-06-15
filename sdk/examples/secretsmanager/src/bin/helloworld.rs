/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use secretsmanager::{Client, Config, Region, SdkError};

use aws_types::region::ProvideRegion;

use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The region. Overrides environment variable AWS_DEFAULT_REGION.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// Specifies the secret's name
    #[structopt(short, long)]
    name: String,

    /// Specifies the secret's value
    #[structopt(short, long)]
    secret_value: String,

    /// Whether to display additional runtime information
    #[structopt(short, long)]
    verbose: bool,
}

/// Creates a secret.
/// # Arguments
///
/// * `-n NAME` - The name of the secret.
/// * `-s SECRET_VALUE` - The secret value.
/// * `[-d DEFAULT-REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() {
    let Opt {
        default_region,
        name,
        secret_value,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    if verbose {
        println!(
            "SecretsManager client version: {}",
            secretsmanager::PKG_VERSION
        );
        println!("Region:                   {:?}", &region);
        println!("Secret name:              {}", name);
        println!("Secret value:             {}", secret_value);

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    let conf = Config::builder().region(region).build();
    let client = Client::from_conf(conf);

    // attempt to create a secret,
    // need to find a better way to handle failure such as ResourceExistsException
    let data = match client
        .create_secret()
        .name(&name)
        .secret_string(&secret_value)
        .send()
        .await
    {
        Ok(secret) => secret,
        Err(SdkError::ServiceError { err, .. }) => match err.kind {
            secretsmanager::error::CreateSecretErrorKind::ResourceExistsError(_) => {
                panic!("This secret already exists!")
            }
            _ => panic!("Secretsmanager Error: {}", err),
        },
        Err(other) => panic!("Failed to create secret: {}", other),
    };
    println!("Created secret {:?} with ARN {:?}", name, data.arn.unwrap());

    //  try and retrieve the secret value we just created
    let retrieved_secret = client
        .get_secret_value()
        .secret_id(name)
        .send()
        .await
        .expect("unable to retrieve secret");

    assert_eq!(retrieved_secret.secret_string.unwrap(), secret_value);
    println!(
        "successfully retrieved secret string that matches the original one we created earlier"
    );
}
