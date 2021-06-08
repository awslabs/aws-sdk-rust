/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#[tokio::main]
async fn main() -> Result<(), rds::Error> {
    let conf = rds::Config::builder()
        .region(rds::Region::new("us-east-1"))
        .build();
    let client = rds::Client::from_conf(conf);
    let result = client.describe_db_instances().send().await?;

    for db_instance in result.db_instances.unwrap_or_default() {
        println!(
            "DB instance identifier: {:?}",
            db_instance
                .db_instance_identifier
                .expect("instance should have identifiers")
        );
        println!(
            "DB instance class: {:?}",
            db_instance
                .db_instance_class
                .expect("instance should have class")
        );
        println!(
            "DB instance engine: {:?}",
            db_instance.engine.expect("instance should have engine")
        );
        println!(
            "DB instance status: {:?}",
            db_instance
                .db_instance_status
                .expect("instance should have status")
        );
        println!(
            "DB instance endpoint: {:?}",
            db_instance.endpoint.expect("instance should have endpoint")
        );
    }

    Ok(())
}
