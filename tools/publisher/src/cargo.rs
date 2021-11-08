/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Module for interacting with Cargo.

mod add_owner;
mod get_owners;
mod publish;
mod yank;

pub use add_owner::AddOwner;
pub use get_owners::GetOwners;
pub use publish::Publish;
pub use yank::Yank;

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::borrow::Cow;
use std::process::{Command, Output};

#[async_trait]
pub trait CargoOperation {
    type Output;

    /// Runs the command asynchronously.
    async fn spawn(&self) -> Result<Self::Output>;

    /// Returns a plan string that can be output to the user to describe the command.
    fn plan(&self) -> Option<Cow<'static, str>>;
}

/// Confirms that cargo exists on the path.
pub fn confirm_installed_on_path() -> Result<()> {
    handle_failure(
        "discover cargo version",
        &Command::new("cargo")
            .arg("version")
            .output()
            .context("cargo is not installed on the PATH")?,
    )
    .context("cargo is not installed on the PATH")
}

/// Returns (stdout, stderr)
fn output_text(output: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn handle_failure(operation_name: &str, output: &Output) -> Result<(), anyhow::Error> {
    if !output.status.success() {
        return Err(capture_error(operation_name, output));
    }
    Ok(())
}

fn capture_error(operation_name: &str, output: &Output) -> anyhow::Error {
    let message = format!(
        "Failed to {name}:\nStatus: {status}\nStdout: {stdout}\nStderr: {stderr}\n",
        name = operation_name,
        status = if let Some(code) = output.status.code() {
            format!("{}", code)
        } else {
            "Killed by signal".to_string()
        },
        stdout = String::from_utf8_lossy(&output.stdout),
        stderr = String::from_utf8_lossy(&output.stderr)
    );
    anyhow::Error::msg(message)
}
