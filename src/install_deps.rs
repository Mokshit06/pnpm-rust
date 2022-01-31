use std::collections::HashMap;
use types::{IncludedDependencies, Project};

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

pub struct InstallDepsOpts {
    workspace: Option<bool>,
    latest: Option<bool>,
    save_workspace_protocol: Option<bool>,
    link_workspace_packages: bool,
    workspace_dir: Option<String>,
    raw_local_config: RawLocalConfig,
    include_direct: Option<IncludedDependencies>,
    all_projects: Option<Vec<Project>>,
}

pub fn install_deps(mut opts: InstallDepsOpts, params: &[&str]) {
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
        let include_direct = opts.include_direct.unwrap_or(IncludedDependencies {
            dependencies: true,
            dev_dependencies: true,
            optional_dependencies: true,
        });
        let force_hoist_pattern =
            opts.raw_local_config.hoist_pattern.is_some() || opts.raw_local_config.hoist.is_some();
        let force_public_hoist_pattern = opts.raw_local_config.shamefully_hoist.is_some()
            || opts.raw_local_config.public_hoist_pattern.is_some();
        let all_projects =
            opts.all_projects
                .unwrap_or(if let Some(workspace_dir) = opts.workspace_dir {
                    find_workspace_packages(&workspace_dir, &opts)
                } else {
                    vec![]
                });
    }
}

fn find_workspace_packages(workspace_dir: &str, opts: &InstallDepsOpts) -> Vec<Project> {
    vec![]
}
