use std::{collections::HashMap, path::Path};

use anyhow::Result;
use find_packages::FindPackagesOpts;
use serde::Deserialize;
use types::{BaseManifest, Project};
mod find_packages;

pub struct WorkspacePackagesOpts {
    engine_strict: Option<bool>,
    node_version: Option<String>,
    patterns: Option<Vec<String>>,
}

pub fn find_workspace_packages(
    workspace_root: &str,
    opts: WorkspacePackagesOpts,
) -> Result<Vec<Project>> {
    let pkgs = find_workspace_packages_no_check(&workspace_root, &opts);

    todo!("check if packages are installable");

    pkgs
}

pub fn find_workspace_packages_no_check(
    workspace_root: &str,
    opts: &WorkspacePackagesOpts,
) -> Result<Vec<Project>> {
    let patterns = opts.patterns.clone().unwrap_or_else(|| {
        require_packages_manifest(&workspace_root)
            .unwrap_or_default()
            .and_then(|manifest| manifest.packages)
            .unwrap_or_default()
    });
    let mut packages = find_packages::find_packages(
        workspace_root,
        Some(FindPackagesOpts {
            include_root: Some(true),
            patterns: Some(patterns),
            ..Default::default()
        }),
    )?;

    packages.sort_by(|project_1, project_2| project_1.dir.partial_cmp(&project_2.dir).unwrap());

    Ok(packages)
}

#[derive(Deserialize)]
pub struct PackagesManifest {
    pub packages: Option<Vec<String>>,
}

fn require_packages_manifest(dir: &str) -> Result<Option<PackagesManifest>> {
    match std::fs::read_to_string(Path::new(dir).join(constants::WORKSPACE_MANIFEST_FILENAME)) {
        Ok(string) => Ok(Some(serde_yaml::from_str(&string)?)),
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => return Ok(None),
            _ => Err(error.into()),
        },
    }
}

pub struct ManifestOnlyPackage {
    pub manifest: BaseManifest,
}

// TODO: currently this function does way too many unecessary allocations
// but right now I'm first trying to make this work
// I'll deal with lifetimes later
pub fn slice_of_workspace_packages_to_map(
    packages: &[ManifestOnlyPackage],
) -> HashMap<String, HashMap<String, &ManifestOnlyPackage>> {
    let mut map = HashMap::new();

    for package in packages {
        match &package.manifest.name {
            Some(name) => {
                if map.get(name).is_none() {
                    map.insert(name.clone(), HashMap::new());
                }
                map.get_mut(name).unwrap().insert(
                    package
                        .manifest
                        .version
                        .clone()
                        .unwrap_or("0.0.0".to_string()),
                    package,
                );
            }
            None => continue,
        };
    }

    map
}
