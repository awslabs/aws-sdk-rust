/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

// Example `RESOURCE_ARN` and `SECRET_ARN`
// const RESOURCE_ARN: &str = "arn:aws:rds:us-west-2:AWS_ACCOUNT:cluster:database-2";
// const SECRET_ARN: &str =
//     "arn:aws:secretsmanager:us-west-2:AWS_ACCOUNT:secret:database2/test/postgres-b8maVb";
const RESOURCE_ARN: &str = "your aurora serverless db cluster resource arn";
const SECRET_ARN: &str = "your secret arn from secret manager";

#[tokio::main]
async fn main() -> Result<(), rdsdata::Error> {
    let conf = rdsdata::Config::builder()
        .region(rdsdata::Region::new("us-west-2"))
        .build();
    let client = rdsdata::Client::from_conf(conf);
    let st = client
        .execute_statement()
        .resource_arn(RESOURCE_ARN)
        .database("postgres") // Do not confuse this with db instance name
        .sql("SELECT * FROM pg_catalog.pg_tables limit 1")
        .secret_arn(SECRET_ARN);

    let result = st.send().await?;

    println!("{:?}", result);
    Ok(())
}
