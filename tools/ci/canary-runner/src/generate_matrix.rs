/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use anyhow::Result;
use crates_io_api::AsyncClient;
use lazy_static::lazy_static;
use serde::Serialize;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct GenerateMatrixOpt {
    #[structopt(short, long)]
    sdk_versions: u8,

    #[structopt(short, long)]
    rust_versions: Vec<String>,
}

lazy_static! {
    static ref CRATES_IO_CLIENT: AsyncClient = AsyncClient::new(
        "AWS_RUST_SDK_PUBLISHER (aws-sdk-rust@amazon.com)",
        Duration::from_secs(1)
    )
    .expect("valid client");
}

#[derive(Debug, Serialize)]
struct Output {
    sdk_version: Vec<String>,
    rust_version: Vec<String>,
}

pub async fn generate_matrix(opt: GenerateMatrixOpt) -> Result<()> {
    let crate_response = CRATES_IO_CLIENT.get_crate("aws-config").await?;
    let output = Output {
        // The versions from the Crates IO client come back in descending order by version number
        sdk_version: crate_response
            .versions
            .into_iter()
            .filter(|v| !v.yanked)
            .map(|v| v.num)
            .take(opt.sdk_versions as usize)
            .collect(),
        rust_version: opt.rust_versions,
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
