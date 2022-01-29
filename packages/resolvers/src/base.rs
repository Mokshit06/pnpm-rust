use std::collections::HashMap;
use types::BaseManifest;

pub struct WorkspacePackage<'a> {
    pub dir: String,
    pub manifest: &'a BaseManifest,
}

pub type WorkspacePackages<'a> = HashMap<String, HashMap<String, WorkspacePackage<'a>>>;

pub enum Resolution {
    TarballResolution {
        tarball: String,
        integrity: Option<String>,
        // needed in some cases to get the auth token
        // sometimes the tarball URL is under a different path
        // and the auth token is specified for the registry only
        registry: Option<String>,
    },
    DirectoryResolution {
        r#type: String,
        directory: String,
    },
    Unknown {
        r#type: String,
    },
}

pub enum ResolvedVia {
    NpmRegistry,
    GitRepository,
    LocalFilesystem,
    Url,
    Other(String),
}

pub struct ResolveResult<'a> {
    pub id: String,
    pub latest: Option<String>,
    pub manifest: Option<&'a BaseManifest>,
    pub normalized_pref: String,
    pub resolution: Resolution,
    pub resolved_via: ResolvedVia,
}
