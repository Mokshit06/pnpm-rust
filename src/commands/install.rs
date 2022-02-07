use crate::Command;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Install {}

impl Command for Install {
    fn exec(&self) {}
}
