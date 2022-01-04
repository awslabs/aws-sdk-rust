/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

// This is the code used by CI to run the canary Lambda.
//
// If running this locally, you'll need to make a clone of awslabs/smithy-rs in
// the aws-sdk-rust project root.
//
// Also consider using the `AWS_PROFILE` and `AWS_REGION` environment variables
// when running this locally.

use anyhow::{bail, Context, Result};
use aws_sdk_cloudwatch as cloudwatch;
use aws_sdk_lambda as lambda;
use aws_sdk_s3 as s3;
use cloudwatch::model::StandardUnit;
use s3::ByteStream;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::{env, path::Path};
use structopt::StructOpt;
use tokio::process::Command;
use tracing::{error, info};

#[derive(StructOpt, Debug)]
pub struct RunOpt {
    #[structopt(long, about = "Version of the SDK to compile the canary against")]
    sdk_version: String,

    #[structopt(
        long,
        about = "The name of the S3 bucket to upload the canary binary bundle to"
    )]
    lambda_code_s3_bucket_name: String,

    #[structopt(
        long,
        about = "The name of the S3 bucket for the canary Lambda to interact with"
    )]
    lambda_test_s3_bucket_name: String,

    #[structopt(long, about = "The ARN of the role that the Lambda will execute as")]
    lambda_execution_role_arn: String,
}

pub async fn run(opt: RunOpt) -> Result<()> {
    let start_time = SystemTime::now();
    let config = aws_config::load_from_env().await;
    let result = run_canary(opt, &config).await;

    let mut metrics = vec![
        (
            "canary-success",
            if result.is_ok() { 1.0 } else { 0.0 },
            StandardUnit::Count,
        ),
        (
            "canary-failure",
            if result.is_ok() { 0.0 } else { 1.0 },
            StandardUnit::Count,
        ),
        (
            "canary-total-time",
            start_time.elapsed().expect("time in range").as_secs_f64(),
            StandardUnit::Seconds,
        ),
    ];
    if let Ok(invoke_time) = result {
        metrics.push((
            "canary-invoke-time",
            invoke_time.as_secs_f64(),
            StandardUnit::Seconds,
        ));
    }

    let cloudwatch_client = cloudwatch::Client::new(&config);
    let mut request_builder = cloudwatch_client
        .put_metric_data()
        .namespace("aws-sdk-rust-canary");
    for metric in metrics {
        request_builder = request_builder.metric_data(
            cloudwatch::model::MetricDatum::builder()
                .metric_name(metric.0)
                .value(metric.1)
                .timestamp(SystemTime::now().into())
                .unit(metric.2)
                .build(),
        );
    }

    info!("Emitting metrics...");
    request_builder
        .send()
        .await
        .context("failed to emit metrics")?;

    result.map(|_| ())
}

async fn run_canary(opt: RunOpt, config: &aws_config::Config) -> Result<Duration> {
    let repo_root = git_root().await?;
    env::set_current_dir(repo_root.join("smithy-rs/tools/ci-cdk/canary-lambda"))
        .context("failed to change working directory")?;

    info!("Generating canary Cargo.toml...");
    generate_cargo_toml(&opt.sdk_version).await?;

    info!("Building the canary...");
    let bundle_path = build_bundle(&opt.sdk_version).await?;
    let bundle_file_name = bundle_path.file_name().unwrap().to_str().unwrap();
    let bundle_name = bundle_path.file_stem().unwrap().to_str().unwrap();

    let s3_client = s3::Client::new(config);
    let lambda_client = lambda::Client::new(config);

    info!("Uploading Lambda code bundle to S3...");
    upload_bundle(
        s3_client,
        &opt.lambda_code_s3_bucket_name,
        bundle_file_name,
        &bundle_path,
    )
    .await?;

    info!(
        "Creating the canary Lambda function named {}...",
        bundle_name
    );
    create_lambda_fn(
        lambda_client.clone(),
        bundle_name,
        bundle_file_name,
        &opt.lambda_execution_role_arn,
        &opt.lambda_code_s3_bucket_name,
        &opt.lambda_test_s3_bucket_name,
    )
    .await?;

    info!("Invoking the canary Lambda...");
    let invoke_start_time = SystemTime::now();
    let invoke_result = invoke_lambda(lambda_client.clone(), bundle_name).await;
    let invoke_time = invoke_start_time.elapsed().expect("time in range");

    info!("Deleting the canary Lambda...");
    delete_lambda_fn(lambda_client, bundle_name).await?;

    invoke_result.map(|_| invoke_time)
}

async fn generate_cargo_toml(sdk_version: &str) -> Result<()> {
    let status = Command::new("./write-cargo-toml.py")
        .arg("--sdk-version")
        .arg(sdk_version)
        .status()
        .await?;
    if !status.success() {
        bail!("Failed to generate canary Cargo.toml");
    }
    Ok(())
}

/// Returns the path to the compiled bundle zip file
async fn build_bundle(sdk_version: &str) -> Result<PathBuf> {
    let output = Command::new("./build-bundle.sh")
        .arg(sdk_version)
        .stderr(std::process::Stdio::inherit())
        .output()
        .await?;
    if !output.status.success() {
        error!(
            "{}",
            std::str::from_utf8(&output.stderr).expect("valid utf-8")
        );
        bail!("Failed to build the canary bundle");
    } else {
        Ok(PathBuf::from(String::from_utf8(output.stdout)?.trim()))
    }
}

async fn upload_bundle(
    s3_client: s3::Client,
    s3_bucket: &str,
    file_name: &str,
    bundle_path: &Path,
) -> Result<()> {
    s3_client
        .put_object()
        .bucket(s3_bucket)
        .key(file_name)
        .body(
            ByteStream::from_path(bundle_path)
                .await
                .context("failed to load bundle file")?,
        )
        .send()
        .await
        .context("failed to upload bundle to S3")?;
    Ok(())
}

async fn create_lambda_fn(
    lambda_client: lambda::Client,
    bundle_name: &str,
    bundle_file_name: &str,
    execution_role: &str,
    code_s3_bucket: &str,
    test_s3_bucket: &str,
) -> Result<()> {
    use lambda::model::*;

    lambda_client
        .create_function()
        .function_name(bundle_name)
        .runtime(Runtime::Providedal2)
        .role(execution_role)
        .handler("aws-sdk-rust-lambda-canary")
        .code(
            FunctionCode::builder()
                .s3_bucket(code_s3_bucket)
                .s3_key(bundle_file_name)
                .build(),
        )
        .publish(true)
        .environment(
            Environment::builder()
                .variables("RUST_BACKTRACE", "1")
                .variables("CANARY_S3_BUCKET_NAME", test_s3_bucket)
                .variables(
                    "CANARY_EXPECTED_TRANSCRIBE_RESULT",
                    "Good day to you transcribe. This is Polly talking to you from the Rust ST K.",
                )
                .build(),
        )
        .timeout(60)
        .send()
        .await
        .context("failed to create canary Lambda function")?;

    let mut attempts = 0;
    let mut state = State::Pending;
    while !matches!(state, State::Active) && attempts < 20 {
        info!("Waiting 1 second for Lambda to become active...");
        tokio::time::sleep(Duration::from_secs(1)).await;
        let configuration = lambda_client
            .get_function_configuration()
            .function_name(bundle_name)
            .send()
            .await
            .context("failed to get Lambda function status")?;
        state = configuration.state.unwrap();
        attempts += 1;
    }
    if !matches!(state, State::Active) {
        bail!("Timed out waiting for canary Lambda to become active");
    }
    Ok(())
}

async fn invoke_lambda(lambda_client: lambda::Client, bundle_name: &str) -> Result<()> {
    use lambda::model::*;
    use lambda::Blob;

    let response = lambda_client
        .invoke()
        .function_name(bundle_name)
        .invocation_type(InvocationType::RequestResponse)
        .log_type(LogType::Tail)
        .payload(Blob::new(&b"{}"[..]))
        .send()
        .await
        .context("failed to invoke the canary Lambda")?;

    if let Some(log_result) = response.log_result {
        info!(
            "Last 4 KB of canary logs:\n----\n{}\n----\n",
            std::str::from_utf8(&base64::decode(&log_result)?)?
        );
    }
    if response.status_code != 200 {
        bail!(
            "Canary failed: {}",
            response
                .function_error
                .as_deref()
                .unwrap_or("<no error given>")
        );
    }
    Ok(())
}

async fn delete_lambda_fn(lambda_client: lambda::Client, bundle_name: &str) -> Result<()> {
    lambda_client
        .delete_function()
        .function_name(bundle_name)
        .send()
        .await
        .context("failed to delete Lambda")?;
    Ok(())
}

async fn git_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .await
        .context("couldn't find repository root")?;
    Ok(PathBuf::from(String::from_utf8(output.stdout)?.trim()))
}
