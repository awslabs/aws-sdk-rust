/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Abstraction of the filesystem to allow for more tests to be added in the future.
#[derive(Clone, Debug)]
pub enum Fs {
    Real,
}

impl Fs {
    /// Reads entire file into `Vec<u8>`
    pub async fn read_file(&self, path: impl AsRef<Path>) -> Result<Vec<u8>> {
        match self {
            Fs::Real => tokio_read_file(path.as_ref()).await,
        }
    }

    /// Writes an entire file from a `&[u8]`
    pub async fn write_file(&self, path: impl AsRef<Path>, contents: &[u8]) -> Result<()> {
        match self {
            Fs::Real => tokio_write_file(path.as_ref(), contents).await,
        }
    }
}

async fn tokio_read_file(path: &Path) -> Result<Vec<u8>> {
    let mut contents = Vec::new();
    let mut file = File::open(path)
        .await
        .with_context(|| format!("failed to open {:?}", path))?;
    file.read_to_end(&mut contents)
        .await
        .with_context(|| format!("failed to read {:?}", path))?;
    Ok(contents)
}

async fn tokio_write_file(path: &Path, contents: &[u8]) -> Result<()> {
    let mut file = File::create(path)
        .await
        .with_context(|| format!("failed to create {:?}", path))?;
    file.write_all(contents)
        .await
        .with_context(|| format!("failed to write {:?}", path))?;
    Ok(())
}
