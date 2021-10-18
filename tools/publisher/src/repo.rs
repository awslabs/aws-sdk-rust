/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Local filesystem git repository discovery. This enables the tool to
//! orient itself despite being run anywhere from within the git repo.

use anyhow::Result;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Git repository containing crates to be published.
#[derive(Debug)]
pub struct Repository {
    pub root: PathBuf,
    pub crates_root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to find {0} repository root")]
    RepositoryRootNotFound(String),
}

/// Attempts to find git repository root from current working directory.
pub fn discover_repository(name: &str, crate_path: &str) -> Result<Repository> {
    let mut current_dir = env::current_dir()?.canonicalize()?;
    let os_name = OsStr::new(name);
    loop {
        if is_git_root(&current_dir) {
            if let Some(file_name) = current_dir.file_name() {
                if os_name == file_name {
                    return Ok(Repository {
                        crates_root: current_dir.join(crate_path),
                        root: current_dir,
                    });
                }
            }
            return Err(Error::RepositoryRootNotFound(name.into()).into());
        } else if !current_dir.pop() {
            return Err(Error::RepositoryRootNotFound(name.into()).into());
        }
    }
}

fn is_git_root(path: &Path) -> bool {
    let path = path.join(".git");
    path.exists() && path.is_dir()
}
