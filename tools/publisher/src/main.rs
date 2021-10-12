/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::subcommand::fix_manifests::subcommand_fix_manifests;
use crate::subcommand::publish::subcommand_publish;
use anyhow::Result;
use clap::{crate_authors, crate_description, crate_name, crate_version};

mod cargo;
mod fs;
mod package;
mod repo;
mod sort;
mod subcommand;

pub const REPO_NAME: &'static str = "aws-sdk-rust";
pub const REPO_CRATE_PATH: &'static str = "sdk";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "error,publisher=info".to_owned()),
        )
        .init();

    let matches = clap_app().get_matches();
    if let Some(_matches) = matches.subcommand_matches("publish") {
        subcommand_publish().await?;
    } else if let Some(_matches) = matches.subcommand_matches("fix-manifests") {
        subcommand_fix_manifests().await?;
    } else {
        clap_app().print_long_help().unwrap();
    }
    Ok(())
}

fn clap_app() -> clap::App<'static, 'static> {
    clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        // In the future, there may be another subcommand for yanking
        .subcommand(
            clap::SubCommand::with_name("fix-manifests")
                .about("fixes path dependencies in manifests to also have version numbers"),
        )
        .subcommand(
            clap::SubCommand::with_name("publish").about("publishes the AWS SDK to crates.io"),
        )
}
