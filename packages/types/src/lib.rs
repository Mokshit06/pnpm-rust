mod package {
    use serde::{Deserialize, Serialize};
    use std::{collections::HashMap, rc::Rc};

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub enum Engines {
        Node(String),
        Npm(String),
        Pnpm(String),
    }

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct BaseManifest {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        //   directories: Option<{,
        //     bin: Option<String>,
        //   }>
        #[serde(skip_serializing_if = "Option::is_none")]
        pub dev_dependencies: Option<Dependencies>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub dependencies: Option<Dependencies>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub optional_dependencies: Option<Dependencies>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub peer_dependencies: Option<Dependencies>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub peer_dependencies_meta: Option<PeerDependenciesMeta>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub dependencies_meta: Option<DependenciesMeta>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bundle_dependencies: Option<Rc<Vec<String>>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bundled_dependencies: Option<Rc<Vec<String>>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub homepage: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub repository: Option<String>,
        //   scripts: Option<PackageScripts>,
        //   config: Option<object>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub engines: Option<Engines>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub cpu: Option<Rc<Vec<String>>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub os: Option<Rc<Vec<String>>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub main: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub module: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub typings: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub types: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub publish_config: Option<PublishConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub readme: Option<String>,
        //   bin: Option<PackageBin>,
    }

    #[derive(Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct DependencyManifest {
        pub name: String,
        pub version: String,
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

    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct PublishConfig {
        directory: Option<String>,
        executable_files: Option<Rc<Vec<String>>>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct DependencyMeta {
        injected: Option<bool>,
        node: Option<String>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

pub struct IncludedDependencies {
    pub optional_dependencies: bool,
    pub dependencies: bool,
    pub dev_dependencies: bool,
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
