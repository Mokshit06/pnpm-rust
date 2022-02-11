use crate::base::{Resolution, ResolveResult, ResolvedVia};
use lazy_static::lazy_static;
use pathdiff;
use read_project_manifest::read_project_manifest_only;
use regex::{Regex, RegexBuilder};
use ssri::{Algorithm, IntegrityOpts};
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub enum PackageType {
    Directory,
    File,
}

pub struct LocalPackageSpec {
    dependency_path: String,
    fetch_spec: String,
    id: String,
    r#type: PackageType,
    normalized_pref: String,
}

pub struct WantedLocalDependency {
    pub pref: String,
    pub injected: Option<bool>,
}

impl WantedLocalDependency {
    pub fn new(pref: String) -> Self {
        Self {
            pref,
            injected: None,
        }
    }
}

pub struct ResolveLocalOpts {
    pub lockfile_dir: Option<String>,
    pub project_dir: String,
}

pub fn resolve_local(
    wanted_dependency: WantedLocalDependency,
    opts: ResolveLocalOpts,
) -> Option<ResolveResult> {
    let spec = parse_pref(
        &wanted_dependency,
        &opts.project_dir,
        opts.lockfile_dir.as_ref().unwrap_or(&opts.project_dir),
    );

    spec.map(|spec| {
        if let PackageType::File = spec.r#type {
            ResolveResult {
                id: spec.id.clone(),
                normalized_pref: spec.normalized_pref,
                latest: None,
                manifest: None,
                resolution: Resolution::TarballResolution {
                    integrity: Some(get_file_integrity(&spec.fetch_spec)),
                    tarball: spec.id,
                    registry: None,
                },
                resolved_via: ResolvedVia::LocalFilesystem,
            }
        } else {
            // handle errors properly (https://github.dev/pnpm/pnpm/blob/334e5340a45d0812750a2bad8fdda5266401c362/packages/local-resolver/src/index.ts#L57)
            let local_dependency_manifest = read_project_manifest_only(&spec.fetch_spec)
                .expect("Error reading project manifest");

            ResolveResult {
                id: spec.id.clone(),
                manifest: Some(local_dependency_manifest),
                latest: None,
                resolution: Resolution::DirectoryResolution {
                    directory: spec.dependency_path,
                    r#type: "directory".to_string(),
                },
                resolved_via: ResolvedVia::LocalFilesystem,
                normalized_pref: spec.normalized_pref,
            }
        }
    })
}

fn get_file_integrity(fetch_spec: &str) -> String {
    IntegrityOpts::new()
        .algorithm(Algorithm::Sha512)
        .chain(fs::read(fetch_spec).unwrap())
        .result()
        .to_string()
}

// i'm not sure why most of these are regex's
// but that's what they were in the original pnpm source code.
// they might be changed to use the std::path
// for better performance and accuracy
lazy_static! {
    static ref IS_WINDOWS: bool = cfg!(windows)
        || env::var("FAKE_WINDOWS").map_or(false, |v| {
            v.parse::<bool>()
                .expect("Couldn't parse `FAKE_WINDOWS` env")
        });
    static ref IS_FILE_SPEC: Regex = Regex::new(if *IS_WINDOWS {
        r"^(?:[.]|~[/]|[/\\]|[a-zA-Z]:)"
    } else {
        r"^(?:[.]|~[/]|[/]|[a-zA-Z]:)"
    })
    .unwrap();
    static ref IS_FILENAME: Regex = RegexBuilder::new(r"[.](?:tgz|tar.gz|tar)$")
        .case_insensitive(true)
        .build()
        .unwrap();
    static ref IS_ABSOLUTE_PATH: Regex = Regex::new(r"^[/]|^[A-Za-z]:").unwrap();
}

fn parse_pref(
    wanted_dependency: &WantedLocalDependency,
    project_dir: &str,
    lockfile_dir: &str,
) -> Option<LocalPackageSpec> {
    let WantedLocalDependency { pref, .. } = &wanted_dependency;
    if pref.starts_with("link:") || pref.starts_with("workspace:") {
        Some(from_local(
            wanted_dependency,
            project_dir,
            lockfile_dir,
            PackageType::Directory,
        ))
    } else if pref.ends_with(".tgz")
        || pref.ends_with(".tar.gz")
        || pref.ends_with(".tar")
        || pref.contains(std::path::MAIN_SEPARATOR)
        || pref.starts_with("file:")
        || IS_FILE_SPEC.is_match(pref)
    {
        let package_type = if IS_FILENAME.is_match(pref) {
            PackageType::File
        } else {
            PackageType::Directory
        };

        Some(from_local(
            wanted_dependency,
            project_dir,
            lockfile_dir,
            package_type,
        ))
    } else if pref.starts_with("path:") {
        panic!("PATH_IS_UNSUPPORTED_PROTOCOL\nLocal dependencies via `path:` protocol are not supported. Use the `link:` protocol for folder dependencies and `file:` for local tarballs")
    } else {
        None
    }
}

fn from_local(
    wanted_dependency: &WantedLocalDependency,
    project_dir: &str,
    lockfile_dir: &str,
    package_type: PackageType,
) -> LocalPackageSpec {
    // rename them to more sensible names
    lazy_static! {
        static ref RE_1: Regex = Regex::new(r"^(file|link|workspace):[/]*([A-Za-z]:)").unwrap();
        static ref RE_2: Regex = Regex::new(r"^(file|link|workspace):(?:[/]*([~./]))?").unwrap();
        static ref RE_3: Regex = Regex::new(r"^~[/]").unwrap();
    }

    let WantedLocalDependency { pref, injected } = wanted_dependency;
    let pref_without_backslash = pref.replace("\\", "/");
    let first_match = RE_1.replace(&pref_without_backslash, "$2");
    let spec = RE_2.replace(&first_match, "$2");
    let protocol = if matches!(package_type, PackageType::Directory) && !injected.unwrap_or(false) {
        "link:"
    } else {
        "file:"
    };
    let fetch_spec: String;
    let normalized_pref: String;
    if RE_3.is_match(&spec) {
        // this is needed for windows and for file:~/foo/bar
        fetch_spec = dirs::home_dir()
            .expect("home_dir not found")
            .join(&spec[2..])
            .to_string_lossy()
            .to_string();
        normalized_pref = format!("{}{}", protocol, spec);
    } else {
        fetch_spec = Path::new(project_dir)
            .join(&*spec)
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string();

        normalized_pref = if Path::new(&*spec).is_absolute() {
            format!("{}{}", protocol, spec)
        } else {
            format!(
                "{}{}",
                protocol,
                relative_path(project_dir, &fetch_spec).to_string_lossy()
            )
        }
    };

    let dependency_path = if injected.unwrap_or(false) {
        relative_path(lockfile_dir, &fetch_spec)
            .to_string_lossy()
            .to_string()
    } else {
        fetch_spec.clone()
    };
    let path_to_join = if !injected.unwrap_or(false)
        && (matches!(package_type, PackageType::Directory) || project_dir == lockfile_dir)
    {
        project_dir
    } else {
        lockfile_dir
    };
    let id = format!(
        "{}{}",
        protocol,
        relative_path(path_to_join, &fetch_spec).to_string_lossy()
    );

    LocalPackageSpec {
        id,
        fetch_spec,
        dependency_path,
        normalized_pref,
        r#type: package_type,
    }
}

// reverse the order of args
fn relative_path<P: AsRef<Path>, S: AsRef<Path>>(base: P, path: S) -> PathBuf {
    pathdiff::diff_paths(path, base).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use path_absolutize::*;
    use pretty_assertions::assert_eq;
    use types::BaseManifest;

    fn dir_name() -> PathBuf {
        std::env::current_dir()
            .unwrap()
            .join("fixtures")
            .join("package")
            .join("nested-package")
    }

    fn dir_name_string() -> String {
        dir_name().to_string_lossy().to_string()
    }

    #[test]
    fn resolve_directory() {
        let resolve_result = resolve_local(
            WantedLocalDependency::new("..".to_string()),
            ResolveLocalOpts {
                lockfile_dir: None,
                project_dir: dir_name_string(),
            },
        );

        assert_matches!(
            resolve_result,
            Some(ResolveResult {
                id,
                normalized_pref,
                manifest: Some(BaseManifest { name: Some(name), .. }),
                resolution: Resolution::DirectoryResolution { directory, r#type },
                ..
            }) => {
                assert_eq!(id, "link:..");
                assert_eq!(normalized_pref, "link:..");
                assert_eq!(name, "@pnpm/local-resolver");
                assert_eq!(directory, dir_name().join("..").absolutize().unwrap().to_string_lossy());
                assert_eq!(r#type, "directory");
            }
        )
    }

    #[test]
    fn resolve_injected_directory() {
        let resolve_result = resolve_local(
            WantedLocalDependency {
                injected: Some(true),
                pref: "..".to_string(),
            },
            ResolveLocalOpts {
                lockfile_dir: None,
                project_dir: dir_name_string(),
            },
        );

        assert_matches!(resolve_result, Some(ResolveResult {
            id,
            normalized_pref,
            manifest: Some(BaseManifest { name: Some(name), .. }),
            resolution: Resolution::DirectoryResolution { directory, r#type },
            ..
        }) => {
            assert_eq!(id, "file:..");
            assert_eq!(normalized_pref, "file:..");
            assert_eq!(name, "@pnpm/local-resolver");
            assert_eq!(directory, "..");
            assert_eq!(r#type, "directory");
        });
    }

    #[test]
    fn resolve_workspace_directory() {
        let resolve_result = resolve_local(
            WantedLocalDependency::new("workspace:..".to_string()),
            ResolveLocalOpts {
                project_dir: dir_name_string(),
                lockfile_dir: None,
            },
        );

        assert_matches!(resolve_result, Some(ResolveResult {
            id,
            normalized_pref,
            manifest: Some(BaseManifest { name: Some(name), .. }),
            resolution: Resolution::DirectoryResolution { directory, r#type },
            ..
        }) => {
            assert_eq!(id, "link:..");
            assert_eq!(normalized_pref, "link:..");
            assert_eq!(name, "@pnpm/local-resolver");
            assert_eq!(directory, dir_name().join("..").absolutize().unwrap().to_string_lossy());
            assert_eq!(r#type,"directory");
        });
    }

    #[test]
    fn resolve_directory_specified_using_the_file_protocol() {
        let resolve_result = resolve_local(
            WantedLocalDependency::new("file:..".to_string()),
            ResolveLocalOpts {
                project_dir: dir_name_string(),
                lockfile_dir: None,
            },
        );

        assert_matches!(resolve_result, Some(ResolveResult {
            id,
            normalized_pref,
            manifest: Some(BaseManifest { name: Some(name), .. }),
            resolution: Resolution::DirectoryResolution { directory, r#type },
            ..
        }) => {
            assert_eq!(id, "link:..");
            assert_eq!(normalized_pref, "link:..");
            assert_eq!(name, "@pnpm/local-resolver");
            assert_eq!(directory, dir_name().join("..").absolutize().unwrap().to_string_lossy());
            assert_eq!(r#type,"directory");
        });
    }

    #[test]
    fn resolve_directoty_specified_using_the_link_protocol() {
        let resolve_result = resolve_local(
            WantedLocalDependency::new("link:..".to_string()),
            ResolveLocalOpts {
                project_dir: dir_name_string(),
                lockfile_dir: None,
            },
        );

        assert_matches!(resolve_result, Some(ResolveResult {
            id,
            normalized_pref,
            manifest: Some(BaseManifest { name: Some(name), .. }),
            resolution: Resolution::DirectoryResolution { directory, r#type },
            ..
        }) => {
            assert_eq!(id, "link:..");
            assert_eq!(normalized_pref, "link:..");
            assert_eq!(name, "@pnpm/local-resolver");
            assert_eq!(directory, dir_name().join("..").absolutize().unwrap().to_string_lossy());
            assert_eq!(r#type,"directory");
        });
    }

    #[test]
    fn resolve_file() {
        let resolve_result = resolve_local(
            WantedLocalDependency::new("./pnpm-local-resolver-0.1.1.tgz".to_string()),
            ResolveLocalOpts {
                lockfile_dir: None,
                project_dir: dir_name_string(),
            },
        );

        assert_eq!(resolve_result, Some(ResolveResult {
            id: "file:pnpm-local-resolver-0.1.1.tgz".to_string(),
            latest: None,
            manifest: None,
            normalized_pref: "file:pnpm-local-resolver-0.1.1.tgz".to_string(),
            resolution: Resolution::TarballResolution {
                integrity: Some("sha512-UHd2zKRT/w70KKzFlj4qcT81A1Q0H7NM9uKxLzIZ/VZqJXzt5Hnnp2PYPb5Ezq/hAamoYKIn5g7fuv69kP258w==".to_string()),
                registry: None,
                tarball: "file:pnpm-local-resolver-0.1.1.tgz".to_string(),
            },
            resolved_via: ResolvedVia::LocalFilesystem,
        }));
    }

    // TODO: implement tests after https://github.com/pnpm/pnpm/blob/main/packages/local-resolver/test/index.ts#L66

    #[test]
    fn resolve_file_when_lockfile_directory_differs_from_the_packages_dir() {
        let resolve_result = resolve_local(
            WantedLocalDependency::new("./pnpm-local-resolver-0.1.1.tgz".to_string()),
            ResolveLocalOpts {
                lockfile_dir: Some(
                    dir_name()
                        .join("..")
                        .absolutize()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                ),
                project_dir: dir_name_string(),
            },
        );

        assert_eq!(resolve_result, Some(ResolveResult {
            id: "file:nested-package/pnpm-local-resolver-0.1.1.tgz".to_string(),
            latest: None,
            manifest: None,
            normalized_pref: "file:pnpm-local-resolver-0.1.1.tgz".to_string(),
            resolution: Resolution::TarballResolution {
                integrity: Some("sha512-UHd2zKRT/w70KKzFlj4qcT81A1Q0H7NM9uKxLzIZ/VZqJXzt5Hnnp2PYPb5Ezq/hAamoYKIn5g7fuv69kP258w==".to_string()),
                registry: None,
                tarball: "file:nested-package/pnpm-local-resolver-0.1.1.tgz".to_string(),
            },
            resolved_via: ResolvedVia::LocalFilesystem,
        }));
    }

    #[test]
    fn resolve_tarball_specified_with_file_protocol() {
        let resolve_result = resolve_local(
            WantedLocalDependency::new("file:./pnpm-local-resolver-0.1.1.tgz".to_string()),
            ResolveLocalOpts {
                lockfile_dir: None,
                project_dir: dir_name_string(),
            },
        );

        assert_eq!(resolve_result, Some(ResolveResult {
            id: "file:pnpm-local-resolver-0.1.1.tgz".to_string(),
            latest: None,
            manifest: None,
            normalized_pref: "file:pnpm-local-resolver-0.1.1.tgz".to_string(),
            resolution: Resolution::TarballResolution {
                integrity: Some("sha512-UHd2zKRT/w70KKzFlj4qcT81A1Q0H7NM9uKxLzIZ/VZqJXzt5Hnnp2PYPb5Ezq/hAamoYKIn5g7fuv69kP258w==".to_string()),
                registry: None,
                tarball: "file:pnpm-local-resolver-0.1.1.tgz".to_string(),
            },
            resolved_via: ResolvedVia::LocalFilesystem,
        }));
    }
}
