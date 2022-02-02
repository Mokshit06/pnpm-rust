use std::path::Path;

use anyhow::Result;
use glob::{glob, glob_with, MatchOptions, Paths};
use globset::{Glob, GlobSetBuilder};
use lazy_static::lazy_static;
use project::Project;
use read_project_manifest::{read_project_manifest, write_project_manifest};
use regex::Regex;
use types::{BaseManifest, ProjectManifest};

const DEFAULT_IGNORE: [&'static str; 4] = [
    "**/node_modules/**",
    "**/bower_components/**",
    "**/test/**",
    "**/tests/**",
];

#[derive(Default)]
pub struct FindPackagesOpts {
    // ignore: Option<Vec<String>>,
    pub include_root: Option<bool>,
    pub patterns: Option<Vec<String>>,
}

pub fn find_packages(root: &str, opts: Option<FindPackagesOpts>) -> Result<Vec<Project>> {
    let opts = opts.unwrap_or(FindPackagesOpts::default());
    // let patterns = normalize_patterns(&opts.patterns.unwrap_or_default());
    let patterns = &opts
        .patterns
        .unwrap_or_else(|| vec![".".to_string(), "**".to_string()]);
    let mut paths = Vec::new();
    for pattern in patterns {
        paths.extend(glob(pattern)?);
    }

    if opts.include_root.unwrap_or(false) {
        paths.extend(glob(".")?);
    }

    let mut manifest_paths = paths
        .iter()
        .map(|manifest_path| Path::new(root).join(manifest_path.as_ref().unwrap()))
        .collect::<Vec<_>>();

    manifest_paths.sort_by(|path_1, path_2| path_1.parent().partial_cmp(&path_2.parent()).unwrap());

    Ok(manifest_paths
        .iter()
        .filter_map(|manifest_path| {
            match read_project_manifest(&manifest_path.to_string_lossy().to_string()) {
                Ok(manifest) => Some(Project {
                    dir: manifest_path
                        .parent()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    manifest: manifest.manifest.expect("manifest not found"),
                    writer_options: manifest.writer_options,
                }),
                Err(_) => None,
            }
        })
        .collect::<Vec<_>>())
}

fn normalize_patterns(patterns: &[String]) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/?$").unwrap();
    }

    let mut normalized_patterns = vec![];
    for pattern in patterns {
        // We should add separate pattern for each extension
        // for some reason, fast-glob is buggy with /package.{json,yaml,json5} pattern
        normalized_patterns.push(RE.replace(pattern, "/package.json").to_string());
        normalized_patterns.push(RE.replace(pattern, "/package.json5").to_string());
        normalized_patterns.push(RE.replace(pattern, "/package.yaml").to_string());
    }

    normalized_patterns
}
