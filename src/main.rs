use clap::{App, AppSettings, ArgGroup, IntoApp, Parser, Subcommand};
use std::process::Command;
mod commands;
use commands::{add, install};
mod install_deps;

#[derive(Parser, Debug)]
#[clap(global_setting(AppSettings::AllowHyphenValues))]
struct Args {
    #[clap(subcommand)]
    command: Option<Commands>,
    /// Pass through commands to npm cli
    #[clap(multiple_values = true)]
    npm: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Installs a package and any packages that it depends on. By default, any new package is installed as a prod dependency add
    Add(add::Add),
    /// Install all dependencies for a project
    Install(install::Install),
    Remove,
    Update,
    Run,
}

impl Commands {
    fn exec(&self) {
        match &self {
            Self::Add(x) => x.exec(),
            Self::Install(x) => x.exec(),
            _ => {}
        }
    }
}

fn main() {
    let args = Args::parse();

    if !args.npm.is_empty() {
        Command::new("npm")
            .args(args.npm)
            .spawn()
            .expect("Error spawning npm task")
            .wait()
            .unwrap();
        std::process::exit(0);
    }

    if let Some(command) = args.command {
    } else {
        Args::into_app().print_help().unwrap();
    }
}
