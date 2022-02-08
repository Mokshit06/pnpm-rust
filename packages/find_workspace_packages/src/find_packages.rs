use anyhow::Result;
use glob::{glob, Pattern};
use lazy_static::lazy_static;
use project::Project;
use rayon::prelude::*;
use read_project_manifest::read_exact_project_manifest;
use regex::Regex;
use std::path::{Path, PathBuf};

lazy_static! {
    static ref DEFAULT_IGNORE: Vec<String> = vec![
        "**/node_modules/**".to_string(),
        "**/bower_components/**".to_string(),
        "**/test/**".to_string(),
        "**/tests/**".to_string(),
    ];
}

#[derive(Default)]
pub struct FindPackagesOpts {
    pub ignore: Option<Vec<String>>,
    pub include_root: Option<bool>,
    pub patterns: Option<Vec<String>>,
}

fn get_manifest_paths<P: AsRef<Path> + Sync>(
    root: P,
    patterns: &[String],
    ignore: &[String],
    include_root: bool,
) -> Result<Vec<PathBuf>> {
    let ignore_pattern = ignore
        .par_iter()
        .map(|path| -> Result<Pattern> {
            Pattern::new(&root.as_ref().join(path).to_string_lossy()).map_err(|err| err.into())
        })
        .collect::<Result<Vec<_>>>()?;

    let mut manifest_paths = patterns
        .par_iter()
        .flat_map(|pattern| {
            glob(&root.as_ref().join(pattern).to_string_lossy())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|path| {
                    !ignore_pattern
                        .par_iter()
                        .any(|ignore| ignore.matches_path(path))
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // I dont think that this part of code is required here
    // So I'll probably remove it later down the line
    if include_root {
        manifest_paths.extend(glob(&normalize_pattern("."))?.filter_map(Result::ok));
    }

    manifest_paths.sort_by(|path_1, path_2| path_1.parent().partial_cmp(&path_2.parent()).unwrap());

    Ok(manifest_paths)
}

pub fn find_packages<P: AsRef<Path> + Sync>(
    root: P,
    opts: Option<FindPackagesOpts>,
) -> Result<Vec<Project>> {
    let opts = opts.unwrap_or_default();
    let patterns = opts
        .patterns
        .unwrap_or_else(|| vec!["**".to_string()])
        .par_iter()
        .map(|pattern| normalize_pattern(pattern))
        .collect::<Vec<_>>();
    let manifest_paths = get_manifest_paths(
        root,
        &patterns,
        opts.ignore.as_deref().unwrap_or(&DEFAULT_IGNORE),
        opts.include_root.unwrap_or(false),
    )?;

    Ok(manifest_paths
        .iter()
        .filter_map(
            |manifest_path| match read_exact_project_manifest(&manifest_path) {
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
            },
        )
        .collect::<Vec<_>>())
}

fn normalize_pattern(pattern: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/?$").unwrap();
    }

    RE.replace(pattern, "/package.json").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use types::BaseManifest;

    #[test]
    fn find_package() {
        let packages = find_packages(Path::new("fixtures").join("one-pkg"), None);

        assert_matches!(packages, Ok(packages) => {
            assert_matches!(&packages[..], [Project {
                dir: _,
                manifest: _,
                ..
            }])
        })
    }

    #[test]
    fn find_packages_by_patterns() {
        let packages = find_packages(
            Path::new("fixtures").join("many-pkgs"),
            Some(FindPackagesOpts {
                patterns: Some(vec!["components/**".to_string()]),
                include_root: None,
                ignore: None,
            }),
        );

        assert_matches!(packages, Ok(packages) => {
            assert_matches!(&packages[..], [
                Project { manifest: BaseManifest { name: Some(name_1), .. }, .. },
                Project { manifest: BaseManifest { name: Some(name_2), .. }, .. }
            ] => {
                assert_eq!(name_1, "component-1");
                assert_eq!(name_2, "component-2");
            })
        });
    }

    #[test]
    fn find_packages_by_default_pattern() {
        let packages = find_packages(
            Path::new("fixtures").join("many-pkgs-2"),
            Some(FindPackagesOpts::default()),
        );

        assert_matches!(packages, Ok(packages) => {
            assert_eq!(packages.len(), 4);
            assert_matches!(&packages[..], [
                Project { manifest: BaseManifest { name: Some(name_1), .. }, .. },
                Project { manifest: BaseManifest { name: Some(name_2), .. }, .. },
                Project { manifest: BaseManifest { name: Some(name_3), .. }, .. },
                Project { manifest: BaseManifest { name: Some(name_4), .. }, .. },
            ] => {
                let mut names = [name_1,name_2,name_3,name_4];
                names.sort();
                assert_eq!(names, ["component-1", "component-2", "foo", "many-pkgs-2"]);
            })
        })
    }

    #[test]
    fn ignore_packages_by_patterns() {
        let packages = find_packages(
            Path::new("fixtures").join("many-pkgs"),
            Some(FindPackagesOpts {
                ignore: Some(vec!["libs/**".to_string()]),
                ..Default::default()
            }),
        );

        assert_matches!(packages, Ok(packages) => {
            assert_eq!(packages.len(), 2);

            assert_matches!(&packages[..], [
                Project { manifest: BaseManifest { name: Some(name_1), .. }, .. },
                Project { manifest: BaseManifest { name: Some(name_2), .. }, .. },
            ] => {
                assert_eq!(name_1, "component-1");
                assert_eq!(name_2, "component-2");
            })
        })
    }
}
