/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Subcommand for fixing manifest dependency version numbers.

use crate::fs::Fs;
use crate::package::{discover_package_manifests, parse_version};
use crate::repo::discover_repository;
use crate::{REPO_CRATE_PATH, REPO_NAME};
use anyhow::{Context, Result};
use cargo_toml::{Dependency, DepsSet};
use semver::Version;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tracing::info;

pub async fn subcommand_fix_manifests() -> Result<()> {
    let repo = discover_repository(REPO_NAME, REPO_CRATE_PATH)?;
    let manifest_paths = discover_package_manifests(&repo.crates_root).await?;
    let mut manifests = read_manifests(Fs::Real, manifest_paths).await?;
    let versions = package_versions(&manifests)?;
    fix_manifests(Fs::Real, &versions, &mut manifests).await?;
    Ok(())
}

struct Manifest {
    path: PathBuf,
    metadata: cargo_toml::Manifest,
}

async fn read_manifests(fs: Fs, manifest_paths: Vec<PathBuf>) -> Result<Vec<Manifest>> {
    let mut result = Vec::new();
    for path in manifest_paths {
        let contents = fs.read_file(&path).await?;
        let metadata = cargo_toml::Manifest::from_slice(&contents)
            .with_context(|| format!("failed to load package manifest for {:?}", &path))?;
        result.push(Manifest { path, metadata });
    }
    Ok(result)
}

fn package_versions(manifests: &Vec<Manifest>) -> Result<BTreeMap<String, Version>> {
    let mut versions = BTreeMap::new();
    for manifest in manifests {
        if let Some(package) = &manifest.metadata.package {
            let version = parse_version(&manifest.path, &package.version)?;
            versions.insert(package.name.clone(), version);
        }
    }
    Ok(versions)
}

fn fix_dep_set(versions: &BTreeMap<String, Version>, dependencies: &mut DepsSet) -> Result<usize> {
    let mut changed = 0;
    for (dep_name, dep) in dependencies {
        changed += match dep {
            Dependency::Simple(_) => 0,
            Dependency::Detailed(detailed) => {
                if detailed.path.is_some() {
                    let version = versions.get(dep_name).ok_or_else(|| {
                        anyhow::Error::msg(format!("version not found for crate {}", dep_name))
                    })?;
                    detailed.version = Some(version.to_string());
                    1
                } else {
                    0
                }
            }
        };
    }
    Ok(changed)
}

fn fix_dep_sets(
    versions: &BTreeMap<String, Version>,
    metadata: &mut cargo_toml::Manifest,
) -> Result<usize> {
    let mut changed = fix_dep_set(versions, &mut metadata.dependencies)?;
    changed += fix_dep_set(versions, &mut metadata.dev_dependencies)?;
    changed += fix_dep_set(versions, &mut metadata.build_dependencies)?;
    Ok(changed)
}

async fn fix_manifests(
    fs: Fs,
    versions: &BTreeMap<String, Version>,
    manifests: &mut Vec<Manifest>,
) -> Result<()> {
    for manifest in manifests {
        let changed = fix_dep_sets(versions, &mut manifest.metadata)?;
        if changed > 0 {
            let contents = toml::to_string(&manifest.metadata)
                .with_context(|| format!("failed to serialize to toml for {:?}", manifest.path))?;
            fs.write_file(&manifest.path, contents.as_bytes()).await?;
            info!("Changed {} dependencies in {:?}.", changed, manifest.path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_dep_sets() {
        let manifest = br#"
            [package]
            name = "test"
            version = "1.2.0-preview"

            [build-dependencies]
            build_something = "1.3"
            local_build_something = { path = "../local_build_something" }

            [dev-dependencies]
            dev_something = "1.1"
            local_dev_something = { path = "../local_dev_something" }

            [dependencies]
            something = "1.0"
            local_something = { path = "../local_something" }
        "#;
        let metadata = cargo_toml::Manifest::from_slice(manifest).unwrap();
        let mut manifest = Manifest {
            path: "test".into(),
            metadata,
        };
        let versions = vec![
            ("local_build_something", "0.2.0"),
            ("local_dev_something", "0.1.0"),
            ("local_something", "1.1.3"),
        ]
        .into_iter()
        .map(|e| (e.0.to_string(), Version::parse(e.1).unwrap()))
        .collect();

        fix_dep_sets(&versions, &mut manifest.metadata).expect("success");

        let actual_deps = toml::Value::try_from(&manifest.metadata.dependencies).unwrap();
        assert_eq!(
            "\
                something = \"1.0\"\n\
                \n\
                [local_something]\n\
                features = []\n\
                optional = false\n\
                path = \"../local_something\"\n\
                version = \"1.1.3\"\n\
            ",
            actual_deps.to_string()
        );

        let actual_dev_deps = toml::Value::try_from(&manifest.metadata.dev_dependencies).unwrap();
        assert_eq!(
            "\
                dev_something = \"1.1\"\n\
                \n\
                [local_dev_something]\n\
                features = []\n\
                optional = false\n\
                path = \"../local_dev_something\"\n\
                version = \"0.1.0\"\n\
            ",
            actual_dev_deps.to_string()
        );

        let actual_build_deps =
            toml::Value::try_from(&manifest.metadata.build_dependencies).unwrap();
        assert_eq!(
            "\
                build_something = \"1.3\"\n\
                \n\
                [local_build_something]\n\
                features = []\n\
                optional = false\n\
                path = \"../local_build_something\"\n\
                version = \"0.2.0\"\n\
            ",
            actual_build_deps.to_string()
        );
    }
}
