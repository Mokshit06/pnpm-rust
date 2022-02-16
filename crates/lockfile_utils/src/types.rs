// import { DependenciesMeta } from '@pnpm/types'
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;
use types::DependenciesMeta;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lockfile {
    #[serde(default)]
    pub importers: HashMap<String, ProjectSnapshot>,
    pub lockfile_version: String,
    pub packages: Option<HashMap<String, PackageSnapshot>>,
    pub never_built_dependencies: Option<Rc<Vec<String>>>,
    pub overrides: Option<HashMap<String, String>>,
    pub package_extensions_checksum: Option<String>,
}

impl Lockfile {
    pub fn new(lockfile_version: String) -> Self {
        Lockfile {
            lockfile_version,
            importers: HashMap::new(),
            packages: None,
            never_built_dependencies: None,
            overrides: None,
            package_extensions_checksum: None,
        }
    }
}

pub type ResolvedDependencies = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    pub specifiers: ResolvedDependencies,
    pub dependencies: Option<ResolvedDependencies>,
    pub optional_dependencies: Option<ResolvedDependencies>,
    pub dev_dependencies: Option<ResolvedDependencies>,
    pub dependencies_meta: Option<DependenciesMeta>,
}

impl ProjectSnapshot {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LockfileResolution {
    RegistryResolution {
        integrity: String,
    },
    GitRepositoryResolution {
        r#type: String,
        repo: String,
        commit: String,
    },
    DirectoryResolution {
        r#type: String,
        directory: String,
    },
    TarballResolution {
        tarball: String,
        integrity: Option<String>,
        // needed in some cases to get the auth token
        // sometimes the tarball URL is under a different path
        // and the auth token is specified for the registry only
        registry: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SnapshotEngines {
    node: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackageSnapshot {
    id: Option<String>,
    dev: Option<bool>,
    optional: Option<bool>,
    requires_build: Option<bool>,
    prepare: Option<bool>,
    has_bin: Option<bool>,
    // name and version are only needed
    // for packages that are hosted not in the npm registry
    name: Option<String>,
    version: Option<String>,
    resolution: LockfileResolution,
    dependencies: Option<ResolvedDependencies>,
    optional_dependencies: Option<ResolvedDependencies>,
    peer_dependencies: Option<HashMap<String, String>>,
    //   peerDependenciesMeta?: {
    //     [name: string]: {
    //       optional: true
    //     }
    //   }
    transitive_peer_dependencies: Option<Vec<String>>,
    bundled_dependencies: Option<Vec<String>>,
    engines: Option<SnapshotEngines>,
    os: Option<Vec<String>>,
    cpu: Option<Vec<String>>,
    deprecated: Option<String>,
}

// pub struct Dependencies {
//   [name: string]: string
// }

// export type PackageBin = string | {[name: string]: string}

// /** @example
//  * {
//  *   "foo": "registry.npmjs.org/foo/1.0.1"
//  * }
//  */
// pub struct ResolvedDependencies {
//   [depName: string]: string
// }
