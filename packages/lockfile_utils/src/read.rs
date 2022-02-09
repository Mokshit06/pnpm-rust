use crate::comver_to_semver;
use crate::git_merge_file::autofix_merge_conflicts;
use crate::types::{Lockfile, PackageSnapshot, ProjectSnapshot, ResolvedDependencies};
use anyhow::{anyhow, bail, Result};
use constants::WANTED_LOCKFILE;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::path::Path;
use std::rc::Rc;
use strip_bom::*;
use types::{DependenciesMeta, DependencyField};

#[derive(Default)]
pub struct ReadLockfileOpts {
    wanted_version: Option<i32>,
    ignore_incompatible: bool,
}

// TODO: change to async function
pub fn read_current_lockfile<P: AsRef<Path> + Debug>(
    virtual_store_dir: P,
    opts: ReadLockfileOpts,
) -> Result<Option<LockfileFile>> {
    let lockfile_path = virtual_store_dir.as_ref().join("lock.yaml");

    read(
        lockfile_path,
        // virtual_store_dir,
        ReadOptions {
            autofix_merge_conflicts: Some(false),
            wanted_version: opts.wanted_version,
            ignore_incompatible: opts.ignore_incompatible,
        },
    )
    .map(|result| result.lockfile)
}

#[derive(Debug)]
pub struct ReadResult {
    lockfile: Option<LockfileFile>,
    had_conflicts: bool,
}

// TODO: change this to just have `lockfile` and `package_snapshot` fields
// and serde flatten it
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LockfileFile {
    #[serde(default)]
    pub importers: HashMap<String, ProjectSnapshot>,
    pub lockfile_version: String,
    pub packages: Option<HashMap<String, PackageSnapshot>>,
    pub never_built_dependencies: Option<Rc<Vec<String>>>,
    pub overrides: Option<HashMap<String, String>>,
    pub package_extensions_checksum: Option<String>,
    pub specifiers: Option<ResolvedDependencies>,
    pub dependencies: Option<ResolvedDependencies>,
    pub optional_dependencies: Option<ResolvedDependencies>,
    pub dev_dependencies: Option<ResolvedDependencies>,
    pub dependencies_meta: Option<DependenciesMeta>,
}

impl LockfileFile {
    pub fn new(lockfile_version: String) -> Self {
        Self {
            lockfile_version,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct ReadOptions {
    autofix_merge_conflicts: Option<bool>,
    wanted_version: Option<i32>,
    ignore_incompatible: bool,
}

pub fn read_wanted_lockfile<P: AsRef<Path> + Debug>(
    pkg_path: P,
    opts: ReadLockfileOpts,
) -> Result<Option<LockfileFile>> {
    let lockfile_path = pkg_path.as_ref().join(WANTED_LOCKFILE);

    read(
        lockfile_path,
        // pkg_path,
        ReadOptions {
            autofix_merge_conflicts: Some(true),
            ignore_incompatible: opts.ignore_incompatible,
            wanted_version: opts.wanted_version,
        },
    )
    .map(|result| result.lockfile)
}

fn read<P: AsRef<Path> + Debug>(
    lockfile_path: P,
    // only used for logging
    // might add it later on
    // _prefix: &str,
    opts: ReadOptions,
) -> Result<ReadResult> {
    let lockfile_raw_content = match std::fs::read_to_string(&lockfile_path) {
        Ok(content) => String::from(content.strip_bom()),
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => bail!(error.to_string()),
            _ => {
                return Ok(ReadResult {
                    lockfile: None,
                    had_conflicts: false,
                })
            }
        },
    };
    let mut had_conflicts = false;
    let mut lockfile = match serde_yaml::from_str::<LockfileFile>(&lockfile_raw_content) {
        Ok(lockfile) => lockfile,
        Err(error) => {
            if !opts.autofix_merge_conflicts.unwrap_or(false) {
                bail!(error.to_string())
            }

            had_conflicts = true;
            autofix_merge_conflicts(&lockfile_raw_content)
        }
    };

    if lockfile.specifiers.is_some() {
        lockfile.importers = HashMap::from_iter([(
            ".".into(),
            ProjectSnapshot {
                specifiers: lockfile.specifiers.take().unwrap(),
                dependencies_meta: lockfile.dependencies_meta.take(),
                ..Default::default()
            },
        )]);

        for dep_type in DependencyField::iterator() {
            let importer = lockfile.importers.get_mut(".").unwrap();
            match dep_type {
                DependencyField::Dependencies => {
                    if lockfile.dependencies.is_some() {
                        importer.dependencies = lockfile.dependencies.take();
                    }
                }
                DependencyField::DevDependencies => {
                    if lockfile.dev_dependencies.is_some() {
                        importer.dev_dependencies = lockfile.dev_dependencies.take();
                    }
                }
                DependencyField::OptionalDependencies => {
                    if lockfile.optional_dependencies.is_some() {
                        importer.optional_dependencies = lockfile.optional_dependencies.take();
                    }
                }
            }
        }
    }

    let lockfile_semver = comver_to_semver(&lockfile.lockfile_version);
    let lockfile_semver = lockfile_semver
        .strip_prefix("v")
        .unwrap_or(&lockfile_semver);

    if opts.wanted_version.is_none()
        || Version::parse(&lockfile_semver).unwrap().major
            == Version::parse(&comver_to_semver(&opts.wanted_version.unwrap().to_string()))
                .unwrap()
                .major
    {
        Ok(ReadResult {
            lockfile: Some(lockfile),
            had_conflicts,
        })
    } else if opts.ignore_incompatible {
        Ok(ReadResult {
            lockfile: None,
            had_conflicts: false,
        })
    } else {
        bail!("{:?}", &lockfile_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use std::collections::HashMap;
    use types::DependencyMeta;

    #[test]
    fn wanted_lockfile() {
        let file = read_wanted_lockfile(
            Path::new("fixtures").join("2"),
            ReadLockfileOpts {
                ignore_incompatible: false,
                ..Default::default()
            },
        );

        assert_matches!(file, Ok(Some(lockfile)) => {
            assert_eq!(lockfile.lockfile_version, "3");
            assert_eq!(lockfile.importers, HashMap::from_iter([
                (".".into(), ProjectSnapshot {
                    specifiers: HashMap::from_iter([
                        ("foo".into(), "1".into())
                    ]),
                    dependencies_meta: Some(HashMap::from_iter([
                        ("foo".into(), DependencyMeta {
                            node: None,
                            injected: Some(true)
                        })
                    ])),
                    ..Default::default()
                })
            ]))
        })
    }

    #[test]
    fn when_lockfile_version_is_string() {
        let lockfile = read_wanted_lockfile(
            Path::new("fixtures").join("4"),
            ReadLockfileOpts {
                ignore_incompatible: false,
                wanted_version: Some(3),
            },
        );

        assert_matches!(lockfile, Ok(Some(lockfile)) => {
            assert_eq!(lockfile.lockfile_version, "v3");
        })
    }

    #[test]
    fn current_lockfile() {
        let lockfile = read_current_lockfile(
            Path::new("fixtures")
                .join("2")
                .join("node_modules")
                .join(".pnpm"),
            ReadLockfileOpts {
                ignore_incompatible: false,
                ..Default::default()
            },
        );

        assert_matches!(lockfile, Ok(Some(lockfile)) => {
            assert_eq!(lockfile.lockfile_version, "3")
        })
    }
}
