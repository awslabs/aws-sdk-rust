/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::process;

use ssm::model::ParameterType;
use ssm::{Client, Config, Region};

use aws_types::region::{self, ProvideRegion};

#[tokio::main]
async fn main() {
    // Determine the region from environment variables or default to us-east-1
    let region = region::default_provider()
        .region()
        .unwrap_or_else(|| Region::new("us-east-1"));

    // Construct a client
    let config = Config::builder().region(region).build();
    let client = Client::from_conf(config);

    // Put an SSM application parameter named `test_parameter_name`
    match client
        .put_parameter()
        .overwrite(true)
        .r#type(ParameterType::String)
        .name("test_parameter_name")
        .value("some_value")
        .description("some description")
        .send()
        .await
    {
        Ok(response) => {
            println!("Success! Parameter now has version: {}", response.version)
        }
        Err(error) => {
            println!("Got an error putting the parameter: {}", error);
            process::exit(1);
        }
    }
}
