/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::cargo::{self, CargoOperation};
use crate::fs::Fs;
use crate::package::{
    continue_batches_from, discover_package_batches, Package, PackageBatch, PackageStats,
};
use crate::repo::discover_repository;
use crate::{REPO_CRATE_PATH, REPO_NAME};
use anyhow::Result;
use dialoguer::Confirm;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::info;

pub async fn subcommand_publish(continue_from: Option<&str>) -> Result<()> {
    // Make sure cargo exists
    cargo::confirm_installed_on_path()?;

    info!("Discovering crates to publish...");
    let repo = discover_repository(REPO_NAME, REPO_CRATE_PATH)?;
    let (mut batches, mut stats) = discover_package_batches(Fs::Real, &repo.crates_root).await?;
    if let Some(continue_from) = continue_from {
        info!(
            "Filtering batches so that publishing starts from {}.",
            continue_from
        );
        continue_batches_from(continue_from, &mut batches, &mut stats)?;
    }
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
                // Only publish if it hasn't been published yet.
                if !is_published(&package).await? {
                    info!("Publishing `{}`...", package.handle);
                    cargo::Publish::new(&package.handle, &package.crate_path)
                        .spawn()
                        .await?;
                    // Sometimes it takes a little bit of time for the new package version
                    // to become available after publish. If we proceed too quickly, then
                    // the next package publish can fail if it depends on this package.
                    wait_for_eventual_consistency(&package).await?;
                }
                drop(permit);
                info!("Successfully published `{}`", package.handle);
                Ok::<_, anyhow::Error>(())
            }));
        }
        for task in tasks {
            task.await??;
        }
    }

    Ok(())
}

async fn is_published(package: &Package) -> Result<bool> {
    cargo::CheckPublished::new(&package.handle, &package.crate_path)
        .spawn()
        .await
}

/// Waits for the given package to show up on crates.io
async fn wait_for_eventual_consistency(package: &Package) -> Result<()> {
    let max_wait_time = 10usize;
    for _ in 0..max_wait_time {
        if !is_published(package).await? {
            tokio::time::sleep(Duration::from_secs(1)).await;
        } else {
            return Ok(());
        }
    }
    if !is_published(package).await? {
        return Err(anyhow::Error::msg(format!(
            "package wasn't found on crates.io {} seconds after publish",
            max_wait_time
        )));
    }
    Ok(())
}

fn confirm_plan(batches: &[PackageBatch], stats: PackageStats) -> Result<()> {
    let mut full_plan = Vec::new();
    for batch in batches {
        for package in batch {
            full_plan.push(
                cargo::Publish::new(&package.handle, &package.crate_path)
                    .plan()
                    .unwrap(),
            );
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
