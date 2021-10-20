/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Module for interacting with Cargo.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

macro_rules! cmd {
    [ $( $x:expr ),* ] => {
        {
            let mut cmd = Cmd::new();
            $(cmd.push($x);)*
            cmd
        }
    };
}

/// Confirms that cargo exists on the path.
pub async fn confirm_installed_on_path() -> Result<()> {
    cmd!["cargo", "--version"]
        .spawn()
        .await
        .context("cargo is not installed on the PATH")?;
    Ok(())
}

/// Returns a `Cmd` that, when spawned, will asynchronously run `cargo publish` in the given crate path.
pub fn publish_task(crate_path: &Path) -> Cmd {
    cmd!["cargo", "publish", "--jobs", "1"].working_dir(crate_path)
}

#[derive(Default)]
pub struct Cmd {
    parts: Vec<String>,
    working_dir: Option<PathBuf>,
}

impl Cmd {
    fn new() -> Cmd {
        Default::default()
    }

    fn push(&mut self, part: impl Into<String>) {
        self.parts.push(part.into());
    }

    fn working_dir(mut self, working_dir: impl AsRef<Path>) -> Self {
        self.working_dir = Some(working_dir.as_ref().into());
        self
    }

    /// Returns a plan string that can be output to the user to describe the command.
    pub fn plan(&self) -> String {
        let mut plan = String::new();
        if let Some(working_dir) = &self.working_dir {
            plan.push_str(&format!("[in {:?}]: ", working_dir));
        }
        plan.push_str(&self.parts.join(" "));
        plan
    }

    /// Runs the command asynchronously.
    pub async fn spawn(mut self) -> Result<Output> {
        let working_dir = self
            .working_dir
            .take()
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        let mut command: Command = self.into();
        tokio::task::spawn_blocking(move || Ok(command.current_dir(working_dir).output()?)).await?
    }
}

impl From<Cmd> for Command {
    fn from(cmd: Cmd) -> Self {
        assert!(!cmd.parts.is_empty());
        let mut command = Command::new(&cmd.parts[0]);
        for i in 1..cmd.parts.len() {
            command.arg(&cmd.parts[i]);
        }
        command
    }
}
