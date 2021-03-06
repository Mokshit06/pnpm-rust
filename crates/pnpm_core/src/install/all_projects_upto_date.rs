use lockfile_utils::satisfies_package_manifest::satisfies_package_manifest;
use lockfile_utils::types::{Lockfile, ProjectSnapshot};
use rayon::prelude::*;
use resolvers::base::WorkspacePackages;
use std::collections::HashMap;
use types::{BaseManifest, DependencyField, ProjectManifest};

pub struct ProjectOptions {
    id: String,
    manifest: ProjectManifest,
    modules_dir: String,
    root_dir: String,
    bins_dir: String,
}

pub struct Options<'a> {
    link_workspace_packages: bool,
    wanted_lockfile: Lockfile,
    workspace_packages: WorkspacePackages<'a>,
}

const X: &str = std::env::consts::OS;
const Y: &str = std::env::consts::ARCH;

fn get_workspace_packages_by_directory<'a>(
    workspace_packages: &'a WorkspacePackages<'a>,
) -> HashMap<&'a str, &'a BaseManifest> {
    let mut workspace_packages_by_directory = HashMap::new();

    for (pkg_name, _) in workspace_packages {
        for (_pkg_version, pkg) in workspace_packages.get(pkg_name).unwrap() {
            workspace_packages_by_directory.insert(pkg.dir.as_str(), pkg.manifest);
        }
    }

    workspace_packages_by_directory
}

pub fn all_projects_are_up_to_date<'a>(projects: &[ProjectOptions], opts: Options<'a>) -> bool {
    let _manifestsByDir = get_workspace_packages_by_directory(&opts.workspace_packages);
    projects.iter().all(|project| {
        let importer = opts
            .wanted_lockfile
            .importers
            .get(project.id.as_str())
            .unwrap();

        has_local_tarball_deps_in_root(importer)
            && satisfies_package_manifest(
                &opts.wanted_lockfile,
                &project.manifest,
                project.id.as_str(),
            )
            && linked_packages_are_up_to_date()
    })
}

fn linked_packages_are_up_to_date() -> bool {
    for dep_field in DependencyField::iterator() {
        // let lockfile_deps =
    }
    todo!();
}

fn has_local_tarball_deps_in_root(importer: &ProjectSnapshot) -> bool {
    [
        &importer.dependencies,
        &importer.dev_dependencies,
        &importer.optional_dependencies,
    ]
    .par_iter()
    .any(|deps| {
        deps.as_ref()
            .unwrap_or(&HashMap::new())
            .par_iter()
            .map(|(_, v)| v)
            .any(|x| ref_is_local_tarball(x.as_str()))
    })
}

fn ref_is_local_tarball(ref_str: &str) -> bool {
    ref_str.starts_with("file:") && ref_str.ends_with(".tgz")
        || ref_str.ends_with(".tar.gz")
        || ref_str.ends_with(".tar")
}
