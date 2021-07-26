/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_auth::provider::{async_provide_credentials_fn, CredentialsError};
use sts::Credentials;

/// Implements a basic version of ProvideCredentials with AWS STS
/// and lists the tables in the region based on those credentials.
#[tokio::main]
async fn main() -> Result<(), dynamodb::Error> {
    tracing_subscriber::fmt::init();
    let client = sts::Client::from_env();

    // NOTE: Do not use this in production! This will grab new credentials for every request.
    // A high quality caching credential provider implementation is in the roadmap.
    let dynamodb_conf = dynamodb::Config::builder()
        .credentials_provider(async_provide_credentials_fn(move || {
            let client = client.clone();
            async move {
                let session_token = client
                    .get_session_token()
                    .send()
                    .await
                    .map_err(|err| CredentialsError::Unhandled(Box::new(err)))?;
                let sts_credentials = session_token
                    .credentials
                    .expect("should include credentials");
                Ok(Credentials::new(
                    sts_credentials.access_key_id.unwrap(),
                    sts_credentials.secret_access_key.unwrap(),
                    sts_credentials.session_token,
                    sts_credentials
                        .expiration
                        .map(|expiry| expiry.to_system_time().expect("sts sent a time < 0")),
                    "Sts",
                ))
            }
        }))
        .build();

    let client = dynamodb::Client::from_conf(dynamodb_conf);
    println!("tables: {:?}", client.list_tables().send().await?);
    Ok(())
}
