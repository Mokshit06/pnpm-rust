use std::env::consts::{ARCH, OS};

pub const WANTED_LOCKFILE: &'static str = "pnpm-lock.yaml";
pub const LOCKFILE_VERSION: f32 = 5.3;

pub const ENGINE_NAME: [&'static str; 4] = [OS, "-", ARCH, "-node"];
pub const LAYOUT_VERSION: i32 = 5;

pub const WORKSPACE_MANIFEST_FILENAME: &'static str = "pnpm-workspace.yaml";
