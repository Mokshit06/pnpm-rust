// import { DependenciesMeta } from '@pnpm/types'
use std::collections::HashMap;
use std::rc::Rc;
use types::DependenciesMeta;

pub struct Lockfile {
    pub importers: HashMap<String, ProjectSnapshot>,
    lockfile_version: f32,
    packages: Option<HashMap<String, PackageSnapshot>>,
    never_built_dependencies: Option<Rc<Vec<String>>>,
    overrides: Option<HashMap<String, String>>,
    package_extensions_checksum: Option<String>,
}

pub type ResolvedDependencies = HashMap<String, String>;

pub struct ProjectSnapshot {
    pub specifiers: ResolvedDependencies,
    pub dependencies: Option<ResolvedDependencies>,
    pub optional_dependencies: Option<ResolvedDependencies>,
    pub dev_dependencies: Option<ResolvedDependencies>,
    pub dependencies_meta: Option<DependenciesMeta>,
}

// pub struct PackageSnapshots {
//   [packagePath: string]: PackageSnapshot
// }

// /**
//  * tarball hosted remotely
//  */
// pub struct TarballResolution {
//   type?: undefined
//   tarball: string
//   integrity?: string
//   // needed in some cases to get the auth token
//   // sometimes the tarball URL is under a different path
//   // and the auth token is specified for the registry only
//   registry?: string
// }

// /**
//  * directory on a file system
//  */
// pub struct DirectoryResolution {
//   type: 'directory'
//   directory: string
// }

// /**
//  * Git repository
//  */
// pub struct GitRepositoryResolution {
//   type: 'git'
//   repo: string
//   commit: string
// }

// export type Resolution =
//   TarballResolution |
//   GitRepositoryResolution |
//   DirectoryResolution

// export type LockfileResolution = Resolution | {
//   integrity: string
// }

pub struct PackageSnapshot {
    id: String,
    //   dev?: true | false
    //   optional?: true
    //   requiresBuild?: true
    //   prepare?: true
    //   hasBin?: true
    //   // name and version are only needed
    //   // for packages that are hosted not in the npm registry
    //   name?: string
    //   version?: string
    //   resolution: LockfileResolution
    //   dependencies?: ResolvedDependencies
    //   optionalDependencies?: ResolvedDependencies
    //   peerDependencies?: {
    //     [name: string]: string
    //   }
    //   peerDependenciesMeta?: {
    //     [name: string]: {
    //       optional: true
    //     }
    //   }
    //   transitivePeerDependencies?: string[]
    //   bundledDependencies?: string[]
    //   engines?: {
    //     node: string
    //   }
    //   os?: string[]
    //   cpu?: string[]
    //   deprecated?: string
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
