/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::cargo;
use crate::fs::Fs;
use crate::package::{discover_package_batches, PackageBatch, PackageStats};
use crate::repo::discover_repository;
use crate::{REPO_CRATE_PATH, REPO_NAME};
use anyhow::Result;
use dialoguer::Confirm;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::info;

const BACKOFF: Duration = Duration::from_millis(30);

pub async fn subcommand_publish() -> Result<()> {
    // Make sure cargo exists
    cargo::confirm_installed_on_path().await?;

    info!("Discovering crates to publish...");
    let repo = discover_repository(REPO_NAME, REPO_CRATE_PATH)?;
    let (batches, stats) = discover_package_batches(Fs::Real, &repo.crates_root).await?;
    info!("Finished crate discovery.");

    // Don't proceed unless the user confirms the plan
    confirm_plan(&batches, stats)?;

    // Use a semaphore to only allow a few concurrent publishes
    let max_concurrency = num_cpus::get_physical();
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    info!(
        "Will publish {} crates in parallel where possible.",
        max_concurrency
    );
    for batch in batches {
        let mut tasks = Vec::new();
        for package in batch {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            tasks.push(tokio::spawn(async move {
                let task = cargo::publish_task(&package.crate_path);
                let plan = task.plan();
                info!("Executing `{}`...", plan);
                let output = task.spawn().await?;
                if !output.status.success() {
                    let already_uploaded_msg = format!(
                        "error: crate version `{}` is already uploaded",
                        package.handle.version
                    );
                    let (stdout, stderr) = (
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr),
                    );
                    if stdout.contains(&already_uploaded_msg)
                        || stderr.contains(&already_uploaded_msg)
                    {
                        info!(
                            "{}-{} has already been published to crates.io.",
                            package.handle.name, package.handle.version
                        );
                    } else {
                        let message = format!(
                            "Cargo publish failed:\nPlan: {}\nStatus: {}\nStdout: {}\nStderr: {}\n",
                            plan,
                            output.status,
                            String::from_utf8_lossy(&output.stdout),
                            String::from_utf8_lossy(&output.stderr)
                        );
                        return Err(anyhow::Error::msg(message));
                    }
                }
                tokio::time::sleep(BACKOFF).await;
                drop(permit);
                info!("Success: `{}`", plan);
                Ok(())
            }));
        }
        for task in tasks {
            task.await??;
        }
    }

    Ok(())
}

fn confirm_plan(batches: &[PackageBatch], stats: PackageStats) -> Result<()> {
    let mut full_plan = Vec::new();
    for batch in batches {
        for package in batch {
            full_plan.push(cargo::publish_task(&package.crate_path).plan());
        }
        full_plan.push("wait".into());
    }

    info!("Publish plan:");
    for item in full_plan {
        println!("  {}", item);
    }
    info!(
        "Will publish {} crates total ({} Smithy runtime, {} AWS runtime, {} AWS SDK).",
        stats.total(),
        stats.smithy_runtime_crates,
        stats.aws_runtime_crates,
        stats.aws_sdk_crates
    );

    if Confirm::new()
        .with_prompt("Continuing will publish to crates.io. Do you wish to continue?")
        .interact()?
    {
        Ok(())
    } else {
        Err(anyhow::Error::msg("aborted"))
    }
}
