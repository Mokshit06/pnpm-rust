use relative_path::RelativePath;
use std::collections::HashMap;
use types::{Graph, IncludedDependencies, Project, ProjectsGraph};

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
    workspace: Option<bool>,
    latest: Option<bool>,
    save_workspace_protocol: Option<bool>,
    link_workspace_packages: bool,
    workspace_dir: Option<String>,
    raw_local_config: RawLocalConfig,
    include_direct: Option<IncludedDependencies>,
    all_projects: Option<Vec<Project>>,
    selected_projects_graph: Option<ProjectsGraph<'a>>,
}

pub fn install_deps<'a>(mut opts: InstallDepsOpts<'a>, params: &[&str]) {
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
        let include_direct = opts.include_direct.unwrap_or_else(|| IncludedDependencies {
            dependencies: true,
            dev_dependencies: true,
            optional_dependencies: true,
        });
        let force_hoist_pattern =
            opts.raw_local_config.hoist_pattern.is_some() || opts.raw_local_config.hoist.is_some();
        let force_public_hoist_pattern = opts.raw_local_config.shamefully_hoist.is_some()
            || opts.raw_local_config.public_hoist_pattern.is_some();
        let all_projects = match opts.all_projects {
            Some(all_projects) => all_projects,
            None => opts
                .workspace_dir
                .as_ref()
                .map(|workspace_dir| find_workspace_packages(&workspace_dir))
                .unwrap_or(vec![]),
        };

        if let Some(workspace_dir) = &opts.workspace_dir {
            if let Some(selected_projects_graph) = match opts.selected_projects_graph {
                Some(graph) => Some(graph),
                None => select_project_by_dir(&all_projects, &opts.dir),
            } {
                // let sequenced_graph = sequence_graph(&selected_projects_graph);
            }
        }
    }
}

fn select_project_by_dir<'a>(
    all_projects: &'a [Project],
    search_dir: &str,
) -> Option<ProjectsGraph<'a>> {
    let project = all_projects.iter().find(|Project { dir, .. }| {
        RelativePath::new(dir)
            .to_path(search_dir)
            .to_string_lossy()
            .to_string()
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

fn find_workspace_packages(workspace_dir: &str) -> Vec<Project> {
    vec![]
}
