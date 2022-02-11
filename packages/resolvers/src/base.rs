use std::collections::HashMap;
use types::BaseManifest;

pub struct WorkspacePackage<'a> {
    pub dir: String,
    pub manifest: &'a BaseManifest,
}

pub struct WantedDependency {
    pub injected: Option<bool>,
    pub pref: Option<String>,
    pub alias: Option<String>,
}

pub struct ResolveOptions<'a> {
    always_try_workspace_packages: Option<bool>,
    default_tag: Option<String>,
    pub project_dir: String,
    pub lockfile_dir: String,
    //   preferredVersions: PreferredVersions
    prefer_workspace_packages: Option<bool>,
    registry: String,
    workspace_packages: Option<WorkspacePackages<'a>>,
}

pub type WorkspacePackages<'a> = HashMap<String, HashMap<String, WorkspacePackage<'a>>>;

/// This type looks almost exactly the same as `LockfileResolution`
/// inside @pnpm/lockfile-utils/types
/// but it was declared as a serperate type in the original package
#[derive(PartialEq, Debug)]
pub enum Resolution {
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
        /// needed in some cases to get the auth token
        /// sometimes the tarball URL is under a different path
        /// and the auth token is specified for the registry only
        registry: Option<String>,
    },
}

#[derive(PartialEq, Debug)]
pub enum ResolvedVia {
    NpmRegistry,
    GitRepository,
    LocalFilesystem,
    Url,
    Other(String),
}

#[derive(PartialEq, Debug)]
pub struct ResolveResult {
    pub id: String,
    pub latest: Option<String>,
    pub manifest: Option<BaseManifest>,
    pub normalized_pref: String,
    pub resolution: Resolution,
    pub resolved_via: ResolvedVia,
}
