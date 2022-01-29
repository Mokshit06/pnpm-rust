use lockfile_utils::satisfies_package_manifest::satisfies_package_manifest;
use lockfile_utils::types::{Lockfile, ProjectSnapshot};
use resolver_base::WorkspacePackages;
use std::collections::HashMap;
use types::{BaseManifest, ProjectManifest};

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

const X: &'static str = std::env::consts::OS;
const Y: &'static str = std::env::consts::ARCH;

fn get_workspace_packages_by_directory<'a>(
    workspace_packages: &'a WorkspacePackages<'a>,
) -> HashMap<&'a str, &'a BaseManifest> {
    let mut workspace_packages_by_directory = HashMap::new();

    for (pkg_name, _) in workspace_packages {
        for (pkg_version, pkg) in workspace_packages.get(pkg_name).unwrap() {
            workspace_packages_by_directory.insert(pkg.dir.as_str(), pkg.manifest);
        }
    }

    workspace_packages_by_directory
}

pub fn all_projects_are_up_to_date<'a>(projects: &[ProjectOptions], opts: Options<'a>) -> bool {
    let manifestsByDir = get_workspace_packages_by_directory(&opts.workspace_packages);

    projects.iter().all(|project| {
        let importer = opts
            .wanted_lockfile
            .importers
            .get(project.id.as_str())
            .unwrap();

        has_local_tarball_deps_in_root(&importer)
            && satisfies_package_manifest(
                &opts.wanted_lockfile,
                &project.manifest,
                project.id.as_str(),
            )
            && linked_packages_are_up_to_date()
    })
}

fn linked_packages_are_up_to_date() -> bool {
    true
}

fn has_local_tarball_deps_in_root(importer: &ProjectSnapshot) -> bool {
    for deps in [
        &importer.dependencies,
        &importer.dev_dependencies,
        &importer.optional_dependencies,
    ] {
        if deps
            .as_ref()
            .unwrap_or(&HashMap::new())
            .iter()
            .map(|(_, v)| v)
            .any(|x| ref_is_local_tarball(x.as_str()))
        {
            return true;
        }
    }

    false
}

fn ref_is_local_tarball(ref_str: &str) -> bool {
    ref_str.starts_with("file:") && ref_str.ends_with(".tgz")
        || ref_str.ends_with(".tar.gz")
        || ref_str.ends_with(".tar")
}
