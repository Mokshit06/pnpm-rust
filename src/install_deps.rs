use crate::recursive::{recursive, CommandFullName, RecursiveOptions};
use anyhow::Result;
use cli_utils::{
    package_is_installable::PackageIsInstallableOpts,
    read_project_manifest::try_read_project_manifest,
};
use find_workspace_packages::{slice_of_workspace_packages_to_map, ManifestOnlyPackage};
use project::{Graph, Project, ProjectsGraph};
use relative_path::RelativePath;
use sort_packages::sequence_graph;
use std::collections::HashMap;
use types::{BaseManifest, IncludedDependencies};

#[derive(Default)]
pub struct RawLocalConfig {
    save_workspace_protocol: Option<bool>,
    user_agent: String,
    shamefully_hoist: Option<bool>,
    public_hoist_pattern: Option<String>,
    hoist_pattern: Option<String>,
    hoist: Option<bool>,
    save_exact: Option<bool>,
    modules_dir: Option<String>,
}

pub struct InstallDepsOpts<'a> {
    dir: String,
    update: Option<bool>,
    workspace: Option<bool>,
    latest: Option<bool>,
    save_workspace_protocol: Option<bool>,
    link_workspace_packages: bool,
    workspace_dir: Option<String>,
    raw_local_config: RawLocalConfig,
    include_direct: Option<IncludedDependencies>,
    all_projects: Option<Vec<Project>>,
    selected_projects_graph: Option<ProjectsGraph<'a>>,
    engine_strict: Option<bool>,
    node_version: Option<String>,
}

pub fn install_deps<'a>(mut opts: InstallDepsOpts<'a>, params: &[&str]) -> Result<()> {
    if opts.workspace.unwrap_or(false) {
        if opts.latest.unwrap_or(false) {
            panic!("BAD_OPTIONS: Cannot use --latest with --workspace simultaneously");
        }
        if opts.workspace.is_none() {
            panic!("WORKSPACE_OPTION_OUTSIDE_WORKSPACE: --workspace can only be used inside a workspace")
        }
        if !opts.link_workspace_packages && !opts.save_workspace_protocol.unwrap_or(false) {
            if let Some(false) = opts.raw_local_config.save_workspace_protocol {
                panic!("BAD_OPTIONS:This workspace has link-workspace-packages turned off, \
                so dependencies are linked from the workspace only when the workspace protocol is used. \
                Either set link-workspace-packages to true or don\'t use the --no-save-workspace-protocol option \
                when running add/update with the --workspace option")
            } else {
                opts.save_workspace_protocol = Some(true)
            }
        }

        // opts['preserveWorkspaceProtocol'] = !opts.linkWorkspacePackages
        let _include_direct = opts.include_direct.unwrap_or(IncludedDependencies {
            dependencies: true,
            dev_dependencies: true,
            optional_dependencies: true,
        });
        let _force_hoist_pattern =
            opts.raw_local_config.hoist_pattern.is_some() || opts.raw_local_config.hoist.is_some();
        let _force_public_hoist_pattern = opts.raw_local_config.shamefully_hoist.is_some()
            || opts.raw_local_config.public_hoist_pattern.is_some();
        let all_projects = match opts.all_projects {
            Some(all_projects) => all_projects,
            None => opts
                .workspace_dir
                .as_ref()
                .map(|workspace_dir| find_workspace_packages(workspace_dir)).unwrap_or_default(),
        };

        if let Some(_workspace_dir) = &opts.workspace_dir {
            if let Some(selected_projects_graph) = match opts.selected_projects_graph {
                Some(graph) => Some(graph),
                None => select_project_by_dir(&all_projects, &opts.dir),
            } {
                let sequenced_graph = sequence_graph(&selected_projects_graph);

                if !sequenced_graph.safe {
                    let _cyclic_dependencies_info = if sequenced_graph.cycles.is_empty() {
                        format!("")
                    } else {
                        sequenced_graph
                            .cycles
                            .iter()
                            .map(|deps| {
                                deps.iter()
                                    .map(|&x| x.clone())
                                    .collect::<Vec<_>>()
                                    .join("-")
                            })
                            .collect::<Vec<String>>()
                            .join("; ")
                    };
                    println!("WARN: there are cyclic workspace dependencies")
                }

                recursive(
                    &all_projects,
                    params,
                    RecursiveOptions {
                        selected_projects_graph,
                    },
                    if opts.update.unwrap_or(false) {
                        CommandFullName::Update
                    } else if params.is_empty() {
                        CommandFullName::Install
                    } else {
                        CommandFullName::Add
                    },
                );
                return Ok(());
            }
        }

        let _params = params
            .iter()
            .filter(|param| !param.is_empty())
            .copied()
            .collect::<Vec<_>>();
        // let dir = opts.dir;
        // .unwrap_or_else(|| {
        //     std::env::current_dir()
        //         .unwrap()
        //         .to_string_lossy()
        //         .to_string()
        // });
        let mut packages = vec![];
        for project in all_projects.iter() {
            packages.push(ManifestOnlyPackage {
                manifest: &project.manifest,
            })
        }
        let _workspace_packages = opts
            .workspace_dir
            .map(|_workspace_dir| slice_of_workspace_packages_to_map(&packages));

        let project_manifest = try_read_project_manifest(
            &opts.dir,
            PackageIsInstallableOpts {
                engine_strict: opts.engine_strict,
                node_version: opts.node_version,
            },
        )?;
        let temp;
        let _manifest = match &project_manifest.manifest {
            Some(manifest) => manifest,
            None => {
                if opts.update.unwrap_or(false) {
                    panic!("NO_IMPORTER_MANIFEST: No package.json found");
                }
                temp = Default::default();
                &temp
            }
        };
        let _write_project_manifest = |updated_manifest: &BaseManifest, force: bool| {
            project_manifest.write_project_manifest(updated_manifest, force)
        };
    }

    Ok(())
}

fn select_project_by_dir<'a>(
    all_projects: &'a [Project],
    search_dir: &str,
) -> Option<ProjectsGraph<'a>> {
    let project = all_projects.iter().find(|Project { dir, .. }| {
        RelativePath::new(dir)
            .to_path(search_dir)
            .to_string_lossy()
            == ""
    });

    project.map(|project| {
        HashMap::from_iter([(
            search_dir.to_string(),
            Graph {
                dependencies: vec![],
                package: project,
            },
        )])
    })
}

fn find_workspace_packages(_workspace_dir: &str) -> Vec<Project> {
    vec![]
}
