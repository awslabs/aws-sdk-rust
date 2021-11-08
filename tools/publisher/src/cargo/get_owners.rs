/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::cargo::{handle_failure, output_text, CargoOperation};
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::borrow::Cow;
use std::process::Command;

pub struct GetOwners<'a> {
    program: &'static str,
    package_name: &'a str,
}

impl<'a> GetOwners<'a> {
    pub fn new(package_name: &'a str) -> GetOwners<'a> {
        GetOwners {
            program: "cargo",
            package_name,
        }
    }
}

#[async_trait]
impl<'a> CargoOperation for GetOwners<'a> {
    type Output = Vec<String>;

    async fn spawn(&self) -> Result<Vec<String>> {
        let mut command = Command::new(self.program);
        command.arg("owner").arg("--list").arg(self.package_name);
        let output = tokio::task::spawn_blocking(move || command.output()).await??;
        handle_failure("get crate owners", &output)?;

        let mut result = Vec::new();
        let (stdout, _) = output_text(&output);
        let line_re = Regex::new(r#"^([\w\d\-_:]+)\s+\([\w\d\s\-_]+\)$"#).unwrap();
        for line in stdout.lines() {
            if let Some(captures) = line_re.captures(line) {
                let user_id = captures.get(1).unwrap().as_str();
                result.push(user_id.to_string());
            } else {
                return Err(anyhow::Error::msg(format!(
                    "unrecognized line in `cargo owner` output: {}",
                    line
                )));
            }
        }
        Ok(result)
    }

    fn plan(&self) -> Option<Cow<'static, str>> {
        None
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_owners_success() {
        let owners = GetOwners {
            program: "./fake_cargo/cargo_owner_list",
            package_name: "aws-sdk-s3",
        }
        .spawn()
        .await
        .unwrap();
        assert_eq!(
            vec![
                "rcoh".to_string(),
                "github:awslabs:rust-sdk-owners".to_string()
            ],
            owners
        );
    }

    #[tokio::test]
    async fn get_owners_failed() {
        let result = GetOwners {
            program: "./fake_cargo/cargo_fails",
            package_name: "aws-sdk-s3",
        }
        .spawn()
        .await;

        assert!(result.is_err(), "expected error, got {:?}", result);
        assert_eq!(
            "Failed to get crate owners:\n\
            Status: 1\n\
            Stdout: some stdout failure message\n\n\
            Stderr: some stderr failure message\n\n",
            format!("{}", result.err().unwrap())
        );
    }
}
