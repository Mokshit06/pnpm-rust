use crate::Command;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Add {}

impl Command for Add {
    fn exec(&self) {}
}
