use anyhow::Result;
use find_packages::{FindPackagesOpts, Project};
mod find_packages;

pub struct WorkspacePackagesOpts {
    engine_strict: Option<bool>,
    node_version: Option<String>,
    patterns: Option<Vec<String>>,
}

pub fn find_workspace_packages(workspace_root: &str, opts: WorkspacePackagesOpts) {
    let pkgs = find_workspace_packages_no_check(&workspace_root, &opts);
}

pub fn find_workspace_packages_no_check(
    workspace_root: &str,
    opts: &WorkspacePackagesOpts,
) -> Result<Vec<Project>> {
    let patterns = opts
        .patterns
        .as_ref()
        .unwrap_or(
            &require_packages_manifest(&workspace_root)
                .and_then(|manifest| manifest.packages)
                .unwrap_or_default(),
        )
        .clone();
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

struct PackagesManifest {
    packages: Option<Vec<String>>,
}

fn require_packages_manifest(dir: &str) -> Option<PackagesManifest> {
    todo!()
}
