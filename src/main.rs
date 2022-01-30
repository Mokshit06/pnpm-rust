use clap::{AppSettings, Parser, Subcommand};
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(global_setting(AppSettings::AllowHyphenValues))]
#[clap(version)]
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
    Add,
    /// Install all dependencies for a project
    Install,
    Remove,
    Update,
    Run,
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

    match args.command {
        _ => {
            println!("works i guess")
        }
    }
}
