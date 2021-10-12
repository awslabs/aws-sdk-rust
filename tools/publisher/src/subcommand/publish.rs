/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::cargo;
use crate::fs::Fs;
use crate::package::{discover_package_batches, PackageBatch};
use crate::repo::discover_repository;
use crate::{REPO_CRATE_PATH, REPO_NAME};
use anyhow::Result;
use dialoguer::Confirm;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::info;

const BACKOFF: Duration = Duration::from_millis(30);
const MAX_CONCURRENCY: usize = 4;

pub async fn subcommand_publish() -> Result<()> {
    // Make sure cargo exists
    cargo::confirm_installed_on_path().await?;

    info!("Discovering crates to publish...");
    let repo = discover_repository(REPO_NAME, REPO_CRATE_PATH)?;
    let batches = discover_package_batches(Fs::Real, &repo.crates_root).await?;
    info!("Crates discovered.");

    // Don't proceed unless the user confirms the plan
    confirm_plan(&batches)?;

    // Use a semaphore to only allow a few concurrent publishes
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENCY));
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
                    let message = format!(
                        "Cargo publish failed:\nPlan: {}\nStatus: {}\nStdout: {}\nStderr: {}\n",
                        plan,
                        output.status,
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    return Err(anyhow::Error::msg(message));
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

fn confirm_plan(batches: &Vec<PackageBatch>) -> Result<()> {
    let mut full_plan = Vec::new();
    for batch in batches {
        for package in batch {
            full_plan.push(cargo::publish_task(&package.crate_path).plan());
        }
        full_plan.push("wait".into());
    }

    println!("Publish plan:");
    for item in full_plan {
        println!("  {}", item);
    }

    if Confirm::new()
        .with_prompt("Continuing will publish to crates.io. Do you wish to continue?")
        .interact()?
    {
        Ok(())
    } else {
        Err(anyhow::Error::msg("aborted"))
    }
}
