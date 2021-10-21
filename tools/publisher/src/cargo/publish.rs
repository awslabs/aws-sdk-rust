/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::cargo::{capture_error, output_text, CargoOperation};
use crate::package::PackageHandle;
use anyhow::Result;
use async_trait::async_trait;
use std::borrow::Cow;
use std::path::Path;
use std::process::Command;
use tracing::info;

pub struct Publish<'a> {
    program: &'static str,
    package_handle: &'a PackageHandle,
    package_path: &'a Path,
}

impl<'a> Publish<'a> {
    pub fn new(package_handle: &'a PackageHandle, package_path: &'a Path) -> Publish<'a> {
        Publish {
            program: "cargo",
            package_handle,
            package_path,
        }
    }
}

#[async_trait]
impl<'a> CargoOperation for Publish<'a> {
    type Output = ();

    async fn spawn(&self) -> Result<()> {
        let mut command = Command::new(self.program);
        command
            .current_dir(self.package_path)
            .env("CARGO_INCREMENTAL", "0") // Disable incremental compilation to reduce disk space used
            .arg("publish")
            .arg("--jobs")
            .arg("1");
        let output = tokio::task::spawn_blocking(move || command.output()).await??;
        if !output.status.success() {
            let (stdout, stderr) = output_text(&output);
            let already_uploaded_msg = format!(
                "error: crate version `{}` is already uploaded",
                self.package_handle.version
            );
            if stdout.contains(&already_uploaded_msg) || stderr.contains(&already_uploaded_msg) {
                info!(
                    "{}-{} has already been published to crates.io.",
                    self.package_handle.name, self.package_handle.version
                );
            } else {
                return Err(capture_error("cargo publish", &output));
            }
        }
        Ok(())
    }

    fn plan(&self) -> Option<Cow<'static, str>> {
        Some(Cow::Owned(format!(
            "[in {:?}]: cargo publish --jobs 1",
            self.package_path
        )))
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use super::*;
    use semver::Version;
    use std::env;

    #[tokio::test]
    async fn publish_succeeds() {
        Publish {
            program: "./fake_cargo/cargo_success",
            package_handle: &PackageHandle::new(
                "aws-sdk-dynamodb",
                Version::parse("0.0.22-alpha").unwrap(),
            ),
            package_path: &env::current_dir().unwrap(),
        }
        .spawn()
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn publish_fails() {
        let result = Publish {
            program: "./fake_cargo/cargo_fails",
            package_handle: &PackageHandle::new(
                "something",
                Version::parse("0.0.22-alpha").unwrap(),
            ),
            package_path: &env::current_dir().unwrap(),
        }
        .spawn()
        .await;
        assert!(result.is_err(), "expected error, got {:?}", result);
        assert_eq!(
            "Failed to cargo publish:\n\
            Status: 1\n\
            Stdout: some stdout failure message\n\n\
            Stderr: some stderr failure message\n\n",
            format!("{}", result.err().unwrap())
        );
    }

    #[tokio::test]
    async fn publish_fails_already_uploaded() {
        Publish {
            program: "./fake_cargo/cargo_publish_already_published",
            package_handle: &PackageHandle::new(
                "aws-sdk-dynamodb",
                Version::parse("0.0.22-alpha").unwrap(),
            ),
            package_path: &env::current_dir().unwrap(),
        }
        .spawn()
        .await
        .unwrap();
    }
}
