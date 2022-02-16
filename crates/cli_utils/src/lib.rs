pub mod package_is_installable {
    use types::BaseManifest;

    #[derive(Default)]
    pub struct PackageIsInstallableOpts {
        pub engine_strict: Option<bool>,
        pub node_version: Option<String>,
    }

    impl PackageIsInstallableOpts {
        pub fn new() -> Self {
            Default::default()
        }
    }

    pub fn package_is_installable(
        _project_dir: &str,
        _manifest: &BaseManifest,
        _opts: PackageIsInstallableOpts,
    ) {
        // let pnpm_version = env!("CARGO_PKG_VERSION");
    }
}

pub mod read_project_manifest {
    use crate::package_is_installable::{package_is_installable, PackageIsInstallableOpts};
    use anyhow::Result;
    use read_project_manifest::{read_project_manifest, ProjectManifest};

    pub fn try_read_project_manifest(
        project_dir: &str,
        opts: PackageIsInstallableOpts,
    ) -> Result<ProjectManifest> {
        let manifest = read_project_manifest(project_dir)?;
        if let Some(manifest) = &manifest.manifest {
            package_is_installable(project_dir, manifest, opts);
        }

        Ok(manifest)
    }
}
