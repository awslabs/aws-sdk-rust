/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::cargo::{handle_failure, CargoOperation};
use anyhow::Result;
use async_trait::async_trait;
use std::borrow::Cow;
use std::process::Command;

pub struct AddOwner<'a> {
    program: &'static str,
    package_name: &'a str,
    owner: &'a str,
}

impl<'a> AddOwner<'a> {
    pub fn new(package_name: &'a str, owner: &'a str) -> AddOwner<'a> {
        AddOwner {
            program: "cargo",
            package_name,
            owner,
        }
    }
}

#[async_trait]
impl<'a> CargoOperation for AddOwner<'a> {
    type Output = ();

    async fn spawn(&self) -> Result<()> {
        let mut command = Command::new(self.program);
        command
            .arg("owner")
            .arg("--add")
            .arg(self.owner)
            .arg(self.package_name);
        let output = tokio::task::spawn_blocking(move || command.output()).await??;
        handle_failure("add owner", &output)?;
        Ok(())
    }

    fn plan(&self) -> Option<Cow<'static, str>> {
        None
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn add_owner_success() {
        AddOwner {
            program: "./fake_cargo/cargo_success",
            package_name: "aws-sdk-s3",
            owner: "github:awslabs:rust-sdk-owners",
        }
        .spawn()
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn get_owners_failed() {
        let result = AddOwner {
            program: "./fake_cargo/cargo_fails",
            package_name: "aws-sdk-s3",
            owner: "github:awslabs:rust-sdk-owners",
        }
        .spawn()
        .await;

        assert!(result.is_err(), "expected error, got {:?}", result);
        assert_eq!(
            "Failed to add owner:\n\
            Status: 1\n\
            Stdout: some stdout failure message\n\n\
            Stderr: some stderr failure message\n\n",
            format!("{}", result.err().unwrap())
        );
    }
}
