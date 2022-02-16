use crate::base::{ResolveOptions, ResolveResult, WantedDependency};
use crate::git;
use crate::local::{self, ResolveLocalOpts, WantedLocalDependency};
// use crate::npm::NpmResolver;
use crate::tarball;
use anyhow::{bail, Result};

struct Resolver {
    // npm_resolver: NpmResolver,
}

impl Resolver {
    pub fn new(// registry: &Registry, credentials: &Credentials, pnpm_options: ResolverFactoryOpts
    ) -> Self {
        // let resolve_from_npm = NpmResolver::new(registry, credentials, &pnpm_options);

        Resolver {
            // npm_resolver: resolve_from_npm,
        }
    }

    pub fn resolve(
        &self,
        wanted_dependency: WantedDependency,
        opts: ResolveOptions,
    ) -> Result<ResolveResult> {
        let resolution = wanted_dependency.pref.as_ref().and_then(|pref| {
            tarball::resolve_tarball(pref)
                .or_else(|| git::resolve_from_git(pref))
                .or_else(|| {
                    local::resolve_local(
                        WantedLocalDependency {
                            pref: pref.to_string(),
                            injected: wanted_dependency.injected,
                        },
                        ResolveLocalOpts {
                            lockfile_dir: Some(opts.lockfile_dir),
                            project_dir: opts.project_dir,
                        },
                    )
                })
        });

        match resolution {
            Some(resolution) => Ok(resolution),
            None => bail!(
                "SPEC_NOT_SUPPORTED_BY_ANY_RESOLVER: {}{} isn't supported by any available resolver.",
                wanted_dependency.alias.map(|alias| format!("{}@", alias)).unwrap_or_else(|| "".to_string()),
                wanted_dependency.pref.unwrap_or_else(|| "".to_string())
            )
        }
    }
}
