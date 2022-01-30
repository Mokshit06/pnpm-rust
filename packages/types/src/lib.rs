mod package {
    use std::{collections::HashMap, rc::Rc};

    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub enum Engines {
        Node(String),
        Npm(String),
        Pnpm(String),
    }

    #[derive(Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct BaseManifest {
        name: String,
        version: String,
        description: Option<String>,
        //   directories: Option<{,
        //     bin: Option<String>,
        //   }>
        pub dev_dependencies: Option<Dependencies>,
        pub dependencies: Option<Dependencies>,
        pub optional_dependencies: Option<Dependencies>,
        pub peer_dependencies: Option<Dependencies>,
        pub peer_dependencies_meta: Option<PeerDependenciesMeta>,
        pub dependencies_meta: Option<DependenciesMeta>,
        bundle_dependencies: Option<Rc<Vec<String>>>,
        bundled_dependencies: Option<Rc<Vec<String>>>,
        homepage: Option<String>,
        repository: Option<String>,
        //   scripts: Option<PackageScripts>,
        //   config: Option<object>,
        engines: Engines,
        cpu: Option<Rc<Vec<String>>>,
        os: Option<Rc<Vec<String>>>,
        main: Option<String>,
        module: Option<String>,
        typings: Option<String>,
        types: Option<String>,
        publish_config: Option<PublishConfig>,
        readme: Option<String>,
        //   bin: Option<PackageBin>,
    }

    pub struct PackageManifest {
        deprecated: Option<String>,
        manifest: BaseManifest,
    }

    pub struct PeerDependencyRules {
        ignore_missing: Option<Rc<Vec<String>>>,
        allowed_versions: Option<HashMap<String, String>>,
    }

    pub struct PackageExtension {
        pub dependencies: Option<Dependencies>,
        pub optional_dependencies: Option<Dependencies>,
        pub peer_dependencies: Option<Dependencies>,
        pub peer_dependencies_meta: Option<PeerDependenciesMeta>,
    }

    pub struct PnpmManifest {
        never_built_dependencies: Option<Rc<Vec<String>>>,
        overrides: Option<HashMap<String, String>>,
        package_extensions: Option<HashMap<String, PackageExtension>>,
        peer_dependency_rules: Option<PeerDependencyRules>,
    }

    pub struct ProjectManifest {
        pub manifest: BaseManifest,
        pnpm: Option<PnpmManifest>,
        private: Option<bool>,
        resolutions: Option<HashMap<String, String>>,
    }

    #[derive(Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    struct PublishConfig {
        directory: Option<String>,
        executable_files: Option<Rc<Vec<String>>>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct DependencyMeta {
        injected: Option<bool>,
        node: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct PeerDependencyMeta {
        optional: Option<bool>,
    }

    pub type DependenciesMeta = HashMap<String, DependencyMeta>;
    pub type Dependencies = HashMap<String, String>;
    pub type PeerDependenciesMeta = HashMap<String, PeerDependencyMeta>;
}

pub use package::*;

#[derive(Clone, Copy)]
pub enum DependencyField {
    OptionalDependencies,
    Dependencies,
    DevDependencies,
}

impl DependencyField {
    pub fn iterator() -> impl Iterator<Item = DependencyField> {
        [
            DependencyField::OptionalDependencies,
            DependencyField::Dependencies,
            DependencyField::DevDependencies,
        ]
        .iter()
        .copied()
    }
}
