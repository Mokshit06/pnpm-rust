use crate::types::Lockfile;
use std::collections::HashMap;
use std::hash::Hash;
use types::{BaseManifest, DependencyField, ProjectManifest};

macro_rules! get_dependency {
    ($val:expr, $dep_field:ident) => {{
        let x = &$val;
        match $dep_field {
            DependencyField::Dependencies => x.dependencies.as_ref(),
            DependencyField::DevDependencies => x.dev_dependencies.as_ref(),
            DependencyField::OptionalDependencies => x.optional_dependencies.as_ref(),
        }
    }};
}

pub fn satisfies_package_manifest(
    lockfile: &Lockfile,
    pkg: &ProjectManifest,
    importer_id: &str,
) -> bool {
    let importer = lockfile.importers.get(importer_id);

    match importer {
        Some(importer) => {
            let BaseManifest {
                dev_dependencies,
                dependencies,
                optional_dependencies,
                ..
            } = &pkg.manifest;

            let dummy_map = HashMap::new();
            let merged_map = dev_dependencies
                .as_ref()
                .unwrap_or(&dummy_map)
                .iter()
                .chain(dependencies.as_ref().unwrap_or(&dummy_map).iter())
                .chain(optional_dependencies.as_ref().unwrap_or(&dummy_map).iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<String, String>>();

            if importer.specifiers != merged_map {
                return false;
            }

            if pkg.manifest.dependencies_meta != importer.dependencies_meta {
                return false;
            }

            for dep_field in DependencyField::iterator() {
                let import_deps = get_dependency!(importer, dep_field).unwrap();
                let pkg_deps = get_dependency!(pkg.manifest, dep_field).unwrap();

                let mut pkg_dep_names: Vec<&str> = vec![];

                match dep_field {
                    DependencyField::OptionalDependencies => {
                        pkg_dep_names =
                            pkg_deps.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>();
                    }
                    DependencyField::DevDependencies => {
                        let mut new_dep_names = vec![];

                        for (k, _) in pkg_deps.iter() {
                            let key = k.as_str();
                            let optional_dependencies = pkg.manifest.optional_dependencies.as_ref();
                            let dependencies = pkg.manifest.dependencies.as_ref();

                            if (optional_dependencies.is_none()
                                || optional_dependencies.unwrap().get(key).is_none())
                                && (dependencies.as_ref().is_none()
                                    || dependencies.unwrap().get(key).is_none())
                            {
                                new_dep_names.push(k)
                            }
                        }
                    }
                    DependencyField::Dependencies => {
                        let optional_dependencies = pkg.manifest.optional_dependencies.as_ref();

                        pkg_dep_names = pkg_deps
                            .iter()
                            .map(|(k, _)| k.as_str())
                            .filter(|&key| {
                                optional_dependencies.is_none()
                                    || optional_dependencies.unwrap().get(key).is_none()
                            })
                            .collect::<Vec<_>>();
                    }
                }

                if pkg_dep_names.len() != import_deps.len()
                // && pkg_dep_names.len() != count_of_non_linked_deps(import_deps)
                {
                    return false;
                }

                for dep_name in pkg_dep_names {
                    if import_deps.contains_key(dep_name)
                        || importer.specifiers.get(dep_name) != pkg_deps.get(dep_name)
                    {
                        return false;
                    }
                }
            }

            true
        }
        None => false,
    }
}
