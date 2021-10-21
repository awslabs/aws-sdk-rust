/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::cargo::{handle_failure, output_text, CargoOperation};
use crate::package::PackageHandle;
use anyhow::{Context, Result};
use async_trait::async_trait;
use regex::Regex;
use semver::Version;
use std::borrow::Cow;
use std::path::Path;
use std::process::Command;

pub struct CheckPublished<'a> {
    program: &'static str,
    package_handle: &'a PackageHandle,
    package_path: &'a Path,
}

impl<'a> CheckPublished<'a> {
    pub fn new(package_handle: &'a PackageHandle, package_path: &'a Path) -> CheckPublished<'a> {
        Self {
            program: "cargo",
            package_handle,
            package_path,
        }
    }
}

#[async_trait]
impl<'a> CargoOperation for CheckPublished<'a> {
    type Output = bool;

    async fn spawn(&self) -> Result<bool> {
        let mut command = Command::new(self.program);
        command
            .current_dir(self.package_path)
            .arg("search")
            .arg(self.package_handle.name.to_string());
        let output = tokio::task::spawn_blocking(move || command.output()).await??;
        handle_failure("check published state", &output)?;
        let (stdout, _) = output_text(&output);

        let line_re = Regex::new(r#"^([a-z0-9\-_]+)\s*=\s*"([a-z0-9\.\-]+)".*$"#).unwrap();
        for line in stdout.lines() {
            if let Some(captures) = line_re.captures(&line) {
                let name = captures.get(1).unwrap().as_str();
                let version = Version::parse(captures.get(2).unwrap().as_str())
                    .context("failed to parse version number from `cargo search` output")
                    .context(format!(
                        "version string: {}",
                        captures.get(1).unwrap().as_str()
                    ))?;
                if name == self.package_handle.name && version == self.package_handle.version {
                    return Ok(true);
                }
            } else {
                return Err(anyhow::Error::msg(format!(
                    "unrecognized line in `cargo search` output: {}",
                    line
                )));
            }
        }
        Ok(false)
    }

    fn plan(&self) -> Option<Cow<'static, str>> {
        None // Not useful to output this operation as part of the plan
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use super::*;
    use semver::Version;
    use std::env;

    #[tokio::test]
    async fn check_published_returns_true() {
        assert!(CheckPublished {
            program: "./fake_cargo/cargo_search_success",
            package_handle: &PackageHandle::new(
                "aws-sdk-dynamodb",
                Version::parse("0.0.22-alpha").unwrap(),
            ),
            package_path: &env::current_dir().unwrap(),
        }
        .spawn()
        .await
        .unwrap());
    }

    #[tokio::test]
    async fn check_published_returns_false() {
        assert!(!CheckPublished {
            program: "./fake_cargo/cargo_search_success",
            package_handle: &PackageHandle::new(
                "definitely-not-published",
                Version::parse("0.0.22-alpha").unwrap(),
            ),
            package_path: &env::current_dir().unwrap(),
        }
        .spawn()
        .await
        .unwrap());
    }

    #[tokio::test]
    async fn check_published_fails() {
        let result = CheckPublished {
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
            "Failed to check published state:\n\
            Status: 1\n\
            Stdout: some stdout failure message\n\n\
            Stderr: some stderr failure message\n\n",
            format!("{}", result.err().unwrap())
        );
    }
}
