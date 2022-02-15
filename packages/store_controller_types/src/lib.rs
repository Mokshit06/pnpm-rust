use resolvers::base::{PreferredVersions, Resolution, WantedDependency, WorkspacePackages};

pub struct StoreController {
    request_package: RequestPackage,
    fetch_package: (),
    import_package: (),
}

impl StoreController {
    pub fn close() {}

    pub fn prune() {}

    pub fn upload(built_pkg_location: &str, opts: UploadOptions) {}
}

pub struct UploadOptions {
    files_index_file: String,
    engine: String,
}

pub struct RequestPackage {}

impl RequestPackage {
    pub fn request(
        wanted_dependency: WantedDependency,
        optional: Option<bool>,
        options: RequestPackageOptions,
    ) {
    }
}

pub struct RequestPackageOptions<'a> {
    always_try_workspace_packages: Option<bool>,
    current_pkg: Option<Package>,
    default_tag: Option<String>,
    download_priority: i32,
    project_dir: String,
    lockfile_dir: String,
    preferred_versions: PreferredVersions,
    prefer_workspace_packages: Option<bool>,
    registry: String,
    side_effects_cache: Option<bool>,
    skip_fetch: Option<bool>,
    update: Option<bool>,
    workspace_packages: Option<WorkspacePackages<'a>>,
}

struct Package {
    id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    resolution: Option<Resolution>,
}
