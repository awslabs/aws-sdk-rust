/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::cargo::{self, CargoOperation};
use crate::fs::Fs;
use crate::package::{discover_package_batches, Package, PackageCategory, PackageHandle};
use crate::repo::discover_repository;
use crate::{REPO_CRATE_PATH, REPO_NAME};
use anyhow::{Context, Result};
use dialoguer::Confirm;
use semver::Version;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::info;

const MAX_CONCURRENCY: usize = 5;

pub async fn subcommand_yank_category(category: &str, version: &str) -> Result<()> {
    let category = match category {
        "aws-runtime" => PackageCategory::AwsRuntime,
        "aws-sdk" => PackageCategory::AwsSdk,
        "smithy-runtime" => PackageCategory::SmithyRuntime,
        _ => {
            return Err(anyhow::Error::msg(format!(
                "unrecognized package category: {}",
                category
            )));
        }
    };
    let version = Version::parse(version).context("failed to parse inputted version number")?;

    // Make sure cargo exists
    cargo::confirm_installed_on_path()?;

    info!("Discovering crates to yank...");
    let repo = discover_repository(REPO_NAME, REPO_CRATE_PATH)?;
    let (batches, _) = discover_package_batches(Fs::Real, &repo.crates_root).await?;
    let packages: Vec<Package> = batches
        .into_iter()
        .flatten()
        .filter(|p| p.category == category)
        .map(|p| {
            Package::new(
                // Replace the version with the version given on the CLI
                PackageHandle::new(p.handle.name, version.clone()),
                p.manifest_path,
                p.local_dependencies,
            )
        })
        .collect();
    info!("Finished crate discovery.");

    // Don't proceed unless the user confirms the plan
    confirm_plan(&packages)?;

    // Use a semaphore to only allow a few concurrent yanks
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENCY));
    info!(
        "Will yank {} crates in parallel where possible.",
        MAX_CONCURRENCY
    );

    let mut tasks = Vec::new();
    for package in packages {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        tasks.push(tokio::spawn(async move {
            info!("Yanking `{}`...", package.handle);
            let result = cargo::Yank::new(&package.handle, &package.crate_path)
                .spawn()
                .await;
            drop(permit);
            info!("Successfully yanked `{}`", package.handle);
            result
        }));
    }
    for task in tasks {
        task.await??;
    }

    Ok(())
}

fn confirm_plan(packages: &[Package]) -> Result<()> {
    info!("Yank plan:");
    for package in packages {
        println!(
            "  {}",
            cargo::Yank::new(&package.handle, &package.crate_path)
                .plan()
                .unwrap()
        );
    }

    if Confirm::new()
        .with_prompt("Continuing will yank crate versions from crates.io. Do you wish to continue?")
        .interact()?
    {
        Ok(())
    } else {
        Err(anyhow::Error::msg("aborted"))
    }
}
