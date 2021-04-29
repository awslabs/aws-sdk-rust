/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#[tokio::main]
async fn main() -> Result<(), qldb::Error> {
    let client = qldb::Client::from_env();
    let result = client.list_ledgers().send().await?;

    if let Some(ledgers) = result.ledgers {
        for ledger in ledgers {
            println!("* {:?}", ledger);
        }

        if result.next_token.is_some() {
            todo!("pagination is not yet demonstrated")
        }
    }

    Ok(())
}
