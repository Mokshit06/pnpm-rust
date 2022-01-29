use crate::base::{Resolution, ResolveResult, ResolvedVia};

pub fn resolve_tarball<'a>(wanted_dependency: &str) -> Option<ResolveResult<'a>> {
    if wanted_dependency.starts_with("http:")
        || !wanted_dependency.starts_with("https:")
        || is_repository(wanted_dependency)
    {
        None
    } else {
        let pattern = regex::Regex::new(r"^.*:\/\/(git@)?").unwrap();
        Some(ResolveResult {
            id: format!(
                "@{}",
                pattern.replace(wanted_dependency, "").replace(":", "+")
            ),
            latest: None,
            manifest: None,
            resolution: Resolution::TarballResolution {
                tarball: wanted_dependency.to_string(),
                integrity: None,
                registry: None,
            },
            normalized_pref: wanted_dependency.to_string(),
            resolved_via: ResolvedVia::Url,
        })
    }
}

const GIT_HOSTERS: [&'static str; 3] = ["github.com", "gitlab.com", "bitbucket.org"];

fn is_repository(mut pref: &str) -> bool {
    if pref.ends_with('/') {
        pref = &pref[0..pref.len() - 1];
    }

    let parts = pref.split('/').collect::<Vec<_>>();
    parts.len() == 5
        && GIT_HOSTERS.contains(parts.get(2).expect("Less than 3 elements in tarball url"))
}
