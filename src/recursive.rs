use project::{Project, ProjectsGraph};

// use anyhow::Result;

pub struct RecursiveOptions<'a> {
    pub selected_projects_graph: ProjectsGraph<'a>,
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
    _params: &[&str],
    opts: RecursiveOptions<'a>,
    _cmd_full_name: CommandFullName,
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

    todo!()
}
