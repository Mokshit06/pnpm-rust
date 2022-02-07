use std::collections::HashMap;

pub struct PackageMeta {
    // TODO: serde rename to 'dist-tag'
    dist_tag: HashMap<String, String>,
    versions: HashMap<String, PackageInRegistry>,
    cached_at: Option<i64>,
}

// maybe convert to a trait
struct PackageMetaCache;

impl PackageMetaCache {
    pub fn get(&self, key: &str) -> Option<PackageMeta> {}

    pub fn set(&self, key: &str) {}
}

pub struct PackageInRegistry {
    manifest: PackageManifest,
    dist: PackageDist,
}

pub struct PackageDist {
    integrity: Option<String>,
    shasum: String,
    tarball: String,
}

pub struct PickPackageOptions {
    auth_header_value: Option<String>,
    preferred_version_selectors: Option<VersionSelectors>,
    registry: String,
    dry_run: bool,
}

pub fn pick_package() {}
