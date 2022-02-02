use crate::base::{Resolution, ResolveResult, ResolvedVia};
use read_project_manifest::read_project_manifest_only;

use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use relative_path::RelativePath;
use ssri;
use ssri::Integrity;
use std::env;
use std::fs;
use std::path::Path;

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
    pref: String,
    injected: Option<bool>,
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
    pub lockfile_dir: String,
    pub project_dir: String,
}

pub fn resolve_local(
    wanted_dependency: WantedLocalDependency,
    opts: ResolveLocalOpts,
) -> Option<ResolveResult> {
    let spec = parse_pref(&wanted_dependency, &opts.project_dir, &opts.lockfile_dir);

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
    Integrity::from(fs::read(fetch_spec).unwrap()).to_string()
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
    let protocol = if matches!(package_type, PackageType::Directory) && injected.unwrap_or(false) {
        "link:"
    } else {
        "file:"
    };
    // this is needed for windows and for file:~/foo/bar
    let fetch_spec: String;
    let normalized_pref: String;
    if RE_3.is_match(&spec) {
        fetch_spec = dirs::home_dir()
            .expect("home_dir not found")
            .join(&spec[2..])
            .to_string_lossy()
            .to_string();
        normalized_pref = format!("{}{}", protocol, spec);
    } else {
        fetch_spec = Path::new(project_dir)
            .join(&*spec)
            .to_string_lossy()
            .to_string();
        normalized_pref = if Path::new(&*spec).is_absolute() {
            format!("{}{}", protocol, spec)
        } else {
            format!(
                "{}{}",
                protocol,
                RelativePath::new(project_dir)
                    .to_path(&fetch_spec)
                    .to_string_lossy()
            )
        }
    };

    let dependency_path = if injected.unwrap_or(false) {
        RelativePath::new(lockfile_dir).to_path(&fetch_spec)
    } else {
        std::env::current_dir().unwrap().join(&fetch_spec)
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
        RelativePath::new(path_to_join)
            .to_path(&fetch_spec)
            .canonicalize()
            .expect("Couldn't resolve path")
            .to_string_lossy()
    );

    LocalPackageSpec {
        id,
        fetch_spec,
        dependency_path: dependency_path.to_string_lossy().to_string(),
        normalized_pref,
        r#type: package_type,
    }
}
