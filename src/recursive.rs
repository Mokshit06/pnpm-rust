use project::{Project, ProjectsGraph};
use sort_packages::sort_packages;
use std::collections::HashMap;
// use anyhow::Result;

pub struct RecursiveOptions<'a> {
    pub selected_projects_graph: ProjectsGraph<'a>,
    pub sort: bool,
}

pub enum CommandFullName {
    Install,
    Add,
    Remove,
    Unlink,
    Update,
    Import,
}

pub fn recursive<'a>(
    all_projects: &[Project],
    params: &[&str],
    opts: RecursiveOptions<'a>,
    cmd_full_name: CommandFullName,
) -> bool {
    if all_projects.is_empty() {
        return false;
    }

    let pkgs = opts
        .selected_projects_graph
        .iter()
        .map(|(_, ws_pkg)| ws_pkg.package)
        .collect::<Vec<_>>();

    if pkgs.is_empty() {
        return false;
    }

    let manifests_by_path = all_projects
        .iter()
        .map(|project| (project.dir.clone(), project))
        .collect::<HashMap<_, _>>();
    let chunks = if opts.sort {
        sort_packages(&opts.selected_projects_graph)
    } else {
        let mut vec = opts
            .selected_projects_graph
            .iter()
            .map(|(k, _)| k)
            .collect::<Vec<_>>();
        vec.sort();
        vec![vec]
    };
    // let store =

    todo!()
}
