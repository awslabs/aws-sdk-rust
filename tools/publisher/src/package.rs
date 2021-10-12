/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Packages, package discovery, and package batching logic.

use crate::fs::Fs;
use crate::sort::dependency_order;
use anyhow::{Context, Result};
use cargo_toml::{Dependency, DepsSet, Manifest};
use semver::Version;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error as StdError;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Information required to identify a package (crate).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PackageHandle {
    pub name: String,
    pub version: Version,
}

impl PackageHandle {
    pub fn new(name: impl Into<String>, version: Version) -> Self {
        Self {
            name: name.into(),
            version,
        }
    }
}

/// Represents a crate (called Package since crate is a reserved word).
#[derive(Debug)]
pub struct Package {
    pub handle: PackageHandle,
    pub crate_path: PathBuf,
    pub manifest_path: PathBuf,
    pub local_dependencies: BTreeSet<PackageHandle>,
}

impl Package {
    pub fn new(
        handle: PackageHandle,
        manifest_path: impl Into<PathBuf>,
        local_dependencies: BTreeSet<PackageHandle>,
    ) -> Self {
        let manifest_path = manifest_path.into();
        Self {
            handle,
            crate_path: manifest_path.parent().unwrap().into(),
            manifest_path,
            local_dependencies,
        }
    }

    /// Returns `true` if this package depends on `other`
    pub fn locally_depends_on(&self, other: &PackageHandle) -> bool {
        self.local_dependencies.contains(other)
    }
}

/// Batch of packages.
pub type PackageBatch = Vec<Package>;

/// Discovers publishable packages in the given directory and returns them as
/// batches that can be published in order.
pub async fn discover_package_batches(fs: Fs, path: impl AsRef<Path>) -> Result<Vec<PackageBatch>> {
    let manifest_paths = discover_package_manifests(path).await?;
    let packages = read_packages(fs, manifest_paths).await?;
    validate_packages(&packages)?;
    batch_packages(packages)
}

type BoxError = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid manifest {0:?}")]
    InvalidManifest(PathBuf),
    #[error(
        "Invalid crate version {1} in {0:?}: {2}. NOTE: All local dependencies \
         must have complete version numbers rather than version requirements."
    )]
    InvalidCrateVersion(PathBuf, String, BoxError),
    #[error("{0:?} missing version in dependency {1}")]
    MissingVersion(PathBuf, String),
    #[error("crate {0} has multiple versions: {1} and {2}")]
    MultipleVersions(String, Version, Version),
}

/// Discovers all Cargo.toml files under the given path with a depth limit of 1.
pub async fn discover_package_manifests(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut manifests = Vec::new();
    let mut read_dir = fs::read_dir(path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let package_path = entry.path();
        if package_path.is_dir() {
            let manifest_path = package_path.join("Cargo.toml");
            if manifest_path.exists() {
                manifests.push(manifest_path);
            }
        }
    }
    Ok(manifests)
}

/// Parses a semver version number and adds additional error context when parsing fails.
pub fn parse_version(manifest_path: &Path, version: &str) -> Result<Version, Error> {
    Version::parse(&version)
        .map_err(|err| Error::InvalidCrateVersion(manifest_path.into(), version.into(), err.into()))
}

fn read_dependencies(path: &Path, dependencies: &DepsSet) -> Result<Vec<PackageHandle>> {
    let mut result = Vec::new();
    for (name, metadata) in dependencies {
        match metadata {
            Dependency::Simple(_) => {}
            Dependency::Detailed(detailed) => {
                if detailed.path.is_some() {
                    let version = detailed
                        .version
                        .as_ref()
                        .map(|version| parse_version(path, &version))
                        .ok_or_else(|| Error::MissingVersion(path.into(), name.into()))??;
                    result.push(PackageHandle::new(name, version));
                }
            }
        }
    }
    Ok(result)
}

fn read_package(path: &Path, manifest_bytes: &[u8]) -> Result<Package> {
    let manifest = Manifest::from_slice(manifest_bytes)
        .with_context(|| format!("failed to load package manifest for {:?}", path))?;
    let package = manifest
        .package
        .ok_or_else(|| Error::InvalidManifest(path.into()))
        .context("crate manifest doesn't have a `[package]` section")?;
    let name = package.name;
    let version = parse_version(path, &package.version)?;
    let handle = PackageHandle { name, version };

    let mut local_dependencies = BTreeSet::new();
    local_dependencies.extend(read_dependencies(path, &manifest.dependencies)?.into_iter());
    local_dependencies.extend(read_dependencies(path, &manifest.dev_dependencies)?.into_iter());
    local_dependencies.extend(read_dependencies(path, &manifest.build_dependencies)?.into_iter());
    Ok(Package::new(handle, path, local_dependencies))
}

/// Validates that all of the publishable crates use consistent version numbers
/// across all of their local dependencies.
fn validate_packages(packages: &Vec<Package>) -> Result<()> {
    let mut versions: BTreeMap<String, Version> = BTreeMap::new();
    let track_version = &mut |handle: &PackageHandle| -> Result<(), Error> {
        if let Some(version) = versions.get(&handle.name) {
            if *version != handle.version {
                Err(Error::MultipleVersions(
                    (&handle.name).into(),
                    versions[&handle.name].clone(),
                    handle.version.clone(),
                ))
            } else {
                Ok(())
            }
        } else {
            versions.insert(handle.name.clone(), handle.version.clone());
            Ok(())
        }
    };
    for package in packages {
        track_version(&package.handle)?;
        for dependency in &package.local_dependencies {
            track_version(dependency)?;
        }
    }

    Ok(())
}

async fn read_packages(fs: Fs, manifest_paths: Vec<PathBuf>) -> Result<Vec<Package>> {
    let mut result = Vec::new();
    for path in &manifest_paths {
        let contents: Vec<u8> = fs.read_file(path).await?;
        result.push(read_package(&path, &contents)?);
    }
    Ok(result)
}

/// Splits the given packages into a list of batches that can be published in order.
/// All of the packages in a given batch can be safely published in parallel.
fn batch_packages(packages: Vec<Package>) -> Result<Vec<PackageBatch>> {
    // Sort packages in order of local dependencies
    let mut packages = dependency_order(packages)?;

    // Discover batches
    let mut batches = Vec::new();
    'outer: while packages.len() > 1 {
        for run in 0..packages.len() {
            let next = &packages[run];
            // If the next package depends on any prior package, then we've discovered the end of the batch
            for index in 0..run {
                let previous = &packages[index];
                if next.locally_depends_on(&previous.handle) {
                    let remaining = packages.split_off(run);
                    let batch = packages;
                    packages = remaining;
                    batches.push(batch);
                    continue 'outer;
                }
            }
        }
        // If the current run is the length of the package vec, then we have exactly one batch left
        break;
    }

    // Push the final batch
    if !packages.is_empty() {
        batches.push(packages);
    }
    Ok(batches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;
    use std::path::PathBuf;

    fn version(version: &str) -> Version {
        Version::parse(version).unwrap()
    }

    #[test]
    fn read_package_success() {
        let manifest = br#"
            [package]
            name = "test"
            version = "1.2.0-preview"

            [build-dependencies]
            build_something = "1.3"
            local_build_something = { version = "0.2.0", path = "../local_build_something" }

            [dev-dependencies]
            dev_something = "1.1"
            local_dev_something = { version = "0.1.0", path = "../local_dev_something" }

            [dependencies]
            something = "1.0"
            local_something = { version = "1.1.3", path = "../local_something" }
        "#;
        let path: PathBuf = "test/Cargo.toml".into();

        let package = read_package(&path, manifest).expect("parse success");
        assert_eq!("test", package.handle.name);
        assert_eq!(version("1.2.0-preview"), package.handle.version);

        let mut expected = BTreeSet::new();
        expected.insert(PackageHandle::new(
            "local_build_something",
            version("0.2.0"),
        ));
        expected.insert(PackageHandle::new("local_dev_something", version("0.1.0")));
        expected.insert(PackageHandle::new("local_something", version("1.1.3")));
        assert_eq!(expected, package.local_dependencies);
    }

    #[test]
    fn read_package_version_requirement_invalid() {
        let manifest = br#"
            [package]
            name = "test"
            version = "1.2.0-preview"

            [dependencies]
            local_something = { version = "1.0", path = "../local_something" }
        "#;
        let path: PathBuf = "test/Cargo.toml".into();

        let error = format!(
            "{}",
            read_package(&path, manifest).err().expect("should fail")
        );
        assert!(
            error.contains("Invalid crate version"),
            "'{}' should contain 'Invalid crate version'",
            error
        );
    }

    fn package(name: &str, dependencies: &[&str]) -> Package {
        Package::new(
            PackageHandle::new(name, Version::parse("1.0.0").unwrap()),
            format!("{}/Cargo.toml", name),
            dependencies
                .iter()
                .map(|d| PackageHandle::new(*d, Version::parse("1.0.0").unwrap()))
                .collect(),
        )
    }

    fn fmt_batches(batches: Vec<PackageBatch>) -> String {
        let mut result = String::new();
        for batch in batches {
            result.push_str(
                &batch
                    .iter()
                    .map(|p| p.handle.name.as_str())
                    .collect::<Vec<&str>>()
                    .join(","),
            );
            result.push(';');
        }
        result
    }

    #[test]
    fn test_batch_packages() {
        assert_eq!("", fmt_batches(batch_packages(vec![]).unwrap()));
        assert_eq!(
            "A;",
            fmt_batches(batch_packages(vec![package("A", &[])]).unwrap())
        );
        assert_eq!(
            "A,B;",
            fmt_batches(batch_packages(vec![package("A", &[]), package("B", &[])]).unwrap())
        );
        assert_eq!(
            "A,B;C;",
            fmt_batches(
                batch_packages(vec![
                    package("C", &["A", "B"]),
                    package("B", &[]),
                    package("A", &[]),
                ])
                .unwrap()
            )
        );
        assert_eq!(
            "A,B;C,F,D;E;",
            fmt_batches(
                batch_packages(vec![
                    package("A", &[]),
                    package("B", &[]),
                    package("C", &["A"]),
                    package("D", &["A", "B"]),
                    package("F", &["B"]),
                    package("E", &["C", "D", "F"]),
                ])
                .unwrap()
            )
        );
    }

    fn pkg_ver(name: &str, version: &str, dependencies: &[(&str, &str)]) -> Package {
        Package::new(
            PackageHandle::new(name, Version::parse(version).unwrap()),
            format!("{}/Cargo.toml", name),
            dependencies
                .iter()
                .map(|p| PackageHandle::new(p.0, Version::parse(p.1).unwrap()))
                .collect(),
        )
    }

    #[test]
    fn test_validate_packages() {
        validate_packages(&vec![
            pkg_ver("A", "1.0.0", &[]),
            pkg_ver("B", "1.1.0", &[]),
            pkg_ver("C", "1.2.0", &[("A", "1.0.0"), ("B", "1.1.0")]),
            pkg_ver("D", "1.3.0", &[("A", "1.0.0")]),
            pkg_ver("F", "1.4.0", &[("B", "1.1.0")]),
            pkg_ver(
                "E",
                "1.5.0",
                &[("C", "1.2.0"), ("D", "1.3.0"), ("F", "1.4.0")],
            ),
        ])
        .expect("success");

        let error = validate_packages(&vec![
            pkg_ver("A", "1.1.0", &[]),
            pkg_ver("B", "1.1.0", &[]),
            pkg_ver("C", "1.2.0", &[("A", "1.1.0"), ("B", "1.1.0")]),
            pkg_ver("D", "1.3.0", &[("A", "1.0.0")]),
            pkg_ver("F", "1.4.0", &[("B", "1.1.0")]),
            pkg_ver(
                "E",
                "1.5.0",
                &[("C", "1.2.0"), ("D", "1.3.0"), ("F", "1.4.0")],
            ),
        ])
        .err()
        .expect("fail");
        assert_eq!(
            "crate A has multiple versions: 1.1.0 and 1.0.0",
            format!("{}", error)
        );
    }
}
