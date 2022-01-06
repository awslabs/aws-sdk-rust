/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use structopt::StructOpt;
use tracing_subscriber::{filter::EnvFilter, prelude::*};

mod generate_matrix;
mod run;

#[derive(StructOpt, Debug)]
#[structopt(name = "canary-runner")]
enum Opt {
    #[structopt(alias = "generate-matrix")]
    GenerateMatrix(generate_matrix::GenerateMatrixOpt),

    #[structopt(alias = "run")]
    Run(run::RunOpt),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warn,canary_runner=info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    let opt = Opt::from_args();
    match opt {
        Opt::GenerateMatrix(subopt) => generate_matrix::generate_matrix(subopt).await,
        Opt::Run(subopt) => run::run(subopt).await,
    }
}
