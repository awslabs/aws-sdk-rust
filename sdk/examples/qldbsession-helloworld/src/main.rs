/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use qldbsession::model::StartSessionRequest;

#[tokio::main]
async fn main() -> Result<(), qldbsession::Error> {
    let client = qldbsession::Client::from_env();
    let result = client
        .send_command()
        .start_session(
            StartSessionRequest::builder()
                // This is the name of the "Getting Started" QLDB ledger. Feel
                // free to change the name!
                .ledger_name("vehicle-registration")
                .build(),
        )
        .send()
        .await?;

    match result.start_session {
        Some(s) => {
            println!("Your session id: {:?}", s.session_token);
        }
        None => unreachable!("a start session will result in an Err or a start session result"),
    }

    Ok(())
}
