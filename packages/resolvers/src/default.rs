// use crate::git::GitResolver;
// use crate::local;
// use crate::npm::NpmResolver;
// use crate::tarball;

// struct Resolver {
//     npm_resolver: NpmResolver,
//     git_resolver: GitResolver,
// }

// impl Resolver {
//     pub fn new(registry: &Registry, credentials: &Credentials, pnpm_options: ResolverFactoryOpts) {
//         let resolve_from_npm = NpmResolver::new(registry, credentials, &pnpm_options);
//         let resolve_from_git = GitResolver::new();

//         Resolver {
//             git_resolver: resolve_from_github,
//             npm_resolver: resolve_from_npm,
//         }
//     }

//     pub fn resolve(&self, wanted_dependency: &str, opts: ()) {
//         let resolution = match self.npm_resolver.resolve(wanted_dependency, opts).as_ref() {
//             Some(resolution) => Some(resolution),
//             None => {
//                 if wanted_dependency.pref {
//                     match tarball::resolve_tarball(wanted_dependency).as_ref() {
//                         Some(r) => Some(r),
//                         None => match self.git_resolver.resolve(wanted_dependency) {
//                             Some(r) => Some(r),
//                             None => local::resolve_local(wanted_dependency, opts),
//                         },
//                     }
//                 }
//             }
//         };

//         match resolution {
//             Some(resolution) => resolution,
//             None => panic!(
//               "SPEC_NOT_SUPPORTED_BY_ANY_RESOLVER: {}{} isn't supported by any available resolver.",
//               wanted_dependency.alias.map(|alias| format!("{}@",alias)).unwrap_or_else(|| "".to_string()),
//               wanted_dependency.perf.unwrap_or("")
//             ),
//         }
//     }
// }
