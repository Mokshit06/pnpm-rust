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
                let import_deps_map = HashMap::new();
                let pkg_deps_map = HashMap::new();
                let import_deps =
                    get_dependency!(importer, dep_field).unwrap_or_else(|| &import_deps_map);
                let pkg_deps =
                    get_dependency!(pkg.manifest, dep_field).unwrap_or_else(|| &pkg_deps_map);

                let pkg_dep_names;

                match dep_field {
                    DependencyField::OptionalDependencies => {
                        pkg_dep_names =
                            pkg_deps.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>();
                    }
                    DependencyField::DevDependencies => {
                        // let mut new_dep_names = vec![];

                        // for (k, _) in pkg_deps.iter() {
                        //     let key = k.as_str();
                        //     let optional_dependencies = pkg.manifest.optional_dependencies.as_ref();
                        //     let dependencies = pkg.manifest.dependencies.as_ref();

                        //     if (optional_dependencies.is_none()
                        //         || optional_dependencies.unwrap().get(key).is_none())
                        //         && (dependencies.as_ref().is_none()
                        //             || dependencies.unwrap().get(key).is_none())
                        //     {
                        //         new_dep_names.push(k)
                        //     }
                        // }
                        pkg_dep_names = pkg_deps
                            .iter()
                            .map(|(k, _)| k)
                            .filter(|dep_name| {
                                let optional_dependencies =
                                    pkg.manifest.optional_dependencies.as_ref();
                                let dependencies = pkg.manifest.dependencies.as_ref();

                                (optional_dependencies.is_none()
                                    || !optional_dependencies.unwrap().contains_key(&**dep_name))
                                    && (dependencies.is_none()
                                        || !dependencies.unwrap().contains_key(&**dep_name))
                            })
                            .map(|key| key.as_str())
                            .collect::<Vec<_>>();
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
                    if !import_deps.contains_key(dep_name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProjectSnapshot;

    impl Default for Lockfile {
        fn default() -> Self {
            Lockfile {
                lockfile_version: 3.to_string(),
                importers: HashMap::new(),
                never_built_dependencies: None,
                overrides: None,
                package_extensions_checksum: None,
                packages: None,
            }
        }
    }

    macro_rules! satisfies {
        ( $fn_name:ident, { $( $lockfile_key:ident: $lockfile_value:expr ),* }, { $( $pkg_field:ident: $pkg_value:expr ),* }, $should_satisfy:expr ) => {
            #[test]
            fn $fn_name() {
                let satisfies = satisfies_package_manifest(&Lockfile {
                    $(
                        $lockfile_key: $lockfile_value,
                    )*
                    ..Default::default()
                }, &ProjectManifest {
                    pnpm: None,
                    private: None,
                    resolutions: None,
                    manifest: BaseManifest {
                        name: Some("foo".into()),
                        version: Some("1.0.0".into()),
                        $(
                            $pkg_field: $pkg_value,
                        )*
                        ..Default::default()
                    }
                }, ".");

                assert_eq!(satisfies, $should_satisfy);
            }
        }
    }

    satisfies!(
        basic_package_manifest,
        {
            importers: HashMap::from_iter([(
                ".".into(),
                ProjectSnapshot {
                    dependencies: Some(HashMap::from_iter([("foo".into(), "1.0.0".into())])),
                    specifiers: HashMap::from_iter([("foo".into(), "^1.0.0".into())]),
                    ..Default::default()
                },
            )])
        },
        {
            dependencies: Some(HashMap::from_iter([("foo".into(), "^1.0.0".into())]))
        },
        true
    );

    satisfies!(
        manifest_with_dev_dependencies,
        {
            importers: HashMap::from_iter([(
                ".".into(),
                ProjectSnapshot {
                    dependencies: Some(HashMap::from_iter([("foo".into(), "1.0.0".into())])),
                    dev_dependencies: Some(HashMap::new()),
                    specifiers: HashMap::from_iter([("foo".into(), "^1.0.0".into())]),
                    ..Default::default()
                },
            )])
        },
        {
            dependencies: Some(HashMap::from_iter([("foo".into(), "^1.0.0".into())]))
        },
        true
    );

    satisfies!(
        manifest_without_dependencies,
        {
            importers: HashMap::from_iter([(
                ".".into(),
                ProjectSnapshot {
                    dev_dependencies: Some(HashMap::from_iter([("foo".into(), "1.0.0".into())])),
                    specifiers: HashMap::from_iter([("foo".into(), "^1.0.0".into())]),
                    ..Default::default()
                },
            )])
        },
        {
            dev_dependencies: Some(HashMap::from_iter([("foo".into(), "^1.0.0".into())]))
        },
        true
    );

    satisfies!(
        manifest_with_optional_dependencies,
        {
            importers: HashMap::from_iter([(
                ".".into(),
                ProjectSnapshot {
                    optional_dependencies: Some(HashMap::from_iter([("foo".into(), "1.0.0".into())])),
                    specifiers: HashMap::from_iter([("foo".into(), "^1.0.0".into())]),
                    ..Default::default()
                },
            )])
        },
        {
            optional_dependencies: Some(HashMap::from_iter([("foo".into(), "^1.0.0".into())]))
        },
        true
    );

    satisfies!(
        lockfile_with_dependencies_and_manifest_with_optional_dependencies,
        {
            importers: HashMap::from_iter([(
                ".".into(),
                ProjectSnapshot {
                    dependencies: Some(HashMap::from_iter([("foo".into(), "1.0.0".into())])),
                    specifiers: HashMap::from_iter([("foo".into(), "^1.0.0".into())]),
                    ..Default::default()
                },
            )])
        },
        {
            optional_dependencies: Some(HashMap::from_iter([("foo".into(), "^1.0.0".into())]))
        },
        false
    );

    satisfies!(
        lockfile_and_manifest_with_different_versions,
        {
            importers: HashMap::from_iter([(
                ".".into(),
                ProjectSnapshot {
                    dependencies: Some(HashMap::from_iter([("foo".into(), "1.0.0".into())])),
                    specifiers: HashMap::from_iter([("foo".into(), "^1.0.0".into())]),
                    ..Default::default()
                },
            )])
        },
        {
            dependencies: Some(HashMap::from_iter([("foo".into(), "^1.1.0".into())]))
        },
        false
    );

    satisfies!(
        lockfile_and_manifest_with_diff_deps,
        {
            importers: HashMap::from_iter([(
                ".".into(),
                ProjectSnapshot {
                    dependencies: Some(HashMap::from_iter([("foo".into(), "1.0.0".into())])),
                    specifiers: HashMap::from_iter([("foo".into(), "^1.0.0".into())]),
                    ..Default::default()
                },
            )])
        },
        {
            dependencies: Some(HashMap::from_iter([
                ("foo".into(), "^1.0.0".into()),
                ("bar".into(), "2.0.0".into())
            ]))
        },
        false
    );

    satisfies!(
        specifiers_and_imports_with_diff_deps,
        {
            importers: HashMap::from_iter([(
                ".".into(),
                ProjectSnapshot {
                    dependencies: Some(HashMap::from_iter([("foo".into(), "1.0.0".into())])),
                    specifiers: HashMap::from_iter([
                        ("foo".into(), "^1.0.0".into()),
                        ("bar".into(), "2.0.0".into())
                    ]),
                    ..Default::default()
                },
            )])
        },
        {
            dependencies: Some(HashMap::from_iter([
                ("foo".into(), "^1.0.0".into()),
                ("bar".into(), "2.0.0".into())
            ]))
        },
        false
    );

    satisfies!(
        dependencies_and_dev_dependencies_in_specifiers,
        {
            importers: HashMap::from_iter([(
                ".".into(),
                ProjectSnapshot {
                    dependencies: Some(HashMap::from_iter([("foo".into(), "1.0.0".into())])),
                    optional_dependencies: Some(HashMap::from_iter([("bar".into(), "2.0.0".into())])),
                    specifiers: HashMap::from_iter([
                        ("foo".into(), "^1.0.0".into()),
                        ("bar".into(), "2.0.0".into())
                    ]),
                    ..Default::default()
                }
            )])
        },
        {
            dependencies: Some(HashMap::from_iter([
                ("foo".into(), "^1.0.0".into()),
                ("bar".into(), "2.0.0".into())
            ])),
            optional_dependencies: Some(HashMap::from_iter([("bar".into(), "2.0.0".into())]))
        },
        true
    );

    // TODO: port tests after https://github.dev/pnpm/pnpm/blob/7ae349cd3f8492a9fc25b42d1d1a81a444e7988b/packages/lockfile-utils/test/satisfiesPackageManifest.ts#L143
}
