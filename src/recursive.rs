use project::{Project, ProjectsGraph};
use std::collections::HashMap;
// use anyhow::Result;

pub struct RecursiveOptions<'a> {
    selected_projects_graph: ProjectsGraph<'a>,
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
    // change to `CommandFullName`
    cmd_full_name: &str,
) -> bool {
    if all_projects.is_empty() {
        return false;
    }

    let pkgs = opts
        .selected_projects_graph
        .iter()
        .map(|(_, ws_pkg)| ws_pkg.package);

    if pkgs.is_empty() {
        return false;
    }

    todo!()
}
