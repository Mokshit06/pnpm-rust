mod package {
    use std::{collections::HashMap, rc::Rc};

    pub enum Engines {
        Node(String),
        Npm(String),
        Pnpm(String),
    }

    pub struct BaseManifest {
        name: String,
        version: String,
        description: Option<String>,
        //   directories: Option<{,
        //     bin: Option<String>,
        //   }>
        dependencies: Option<Dependencies>,
        devDependencies: Option<Dependencies>,
        optionalDependencies: Option<Dependencies>,
        peerDependencies: Option<Dependencies>,
        peerDependenciesMeta: Option<PeerDependenciesMeta>,
        dependenciesMeta: Option<DependenciesMeta>,
        bundleDependencies: Option<Rc<Vec<String>>>,
        bundledDependencies: Option<Rc<Vec<String>>>,
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
        publishConfig: Option<PublishConfig>,
        readme: Option<String>,
        //   bin: Option<PackageBin>,
    }

    pub struct PackageManifest {
        deprecated: Option<String>,
        manifest: BaseManifest,
    }

    pub struct PnpmManifest {}

    pub struct ProjectManifest {
        manifest: BaseManifest,
        pnpm: Option<PnpmManifest>,
    }

    struct PublishConfig {
        directory: Option<String>,
        executableFiles: Option<Rc<Vec<String>>>,
    }

    pub struct DependencyMeta {
        injected: Option<bool>,
        node: Option<String>,
    }

    pub struct PeerDependencyMeta {
        optional: Option<bool>,
    }

    pub type DependenciesMeta = HashMap<String, DependencyMeta>;
    pub type Dependencies = HashMap<String, String>;
    pub type PeerDependenciesMeta = HashMap<String, PeerDependencyMeta>;
}

pub use package::*;

pub enum DependencyFields {
    OptionalDependencies,
    Dependencies,
    DevDependencies,
}
