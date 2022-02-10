mod parse_pref;
use crate::base::{Resolution, ResolveResult, ResolvedVia};
use anyhow::{bail, Result};
use lazy_static::lazy_static;
use parse_pref::parse_pref;
use rayon::prelude::*;
use regex::Regex;
use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::process::Command;

pub fn resolve_from_git(pref: &str) -> Option<ResolveResult> {
    lazy_static! {
        static ref RE_1: Regex = Regex::new(r"^.*://(git@)?").unwrap();
        static ref RE_2: Regex = Regex::new(r":").unwrap();
        static ref RE_3: Regex = Regex::new(r"\.git$").unwrap();
    }

    let parsed_spec = parse_pref(pref);

    match parsed_spec {
        Some(mut parsed_spec) => {
            let pref = match &parsed_spec.git_committish {
                Some(committish) if committish.is_empty() => "HEAD",
                Some(committish) => committish,
                None => "HEAD",
            };
            let commit = match resolve_ref(
                &parsed_spec.fetch_spec,
                pref,
                parsed_spec.git_range.as_deref(),
            ) {
                Ok(commit) => commit,
                Err(err) => panic!("{}", err),
            };
            let mut resolution = None;

            if let Some(hosted) = &mut parsed_spec.hosted {
                if !is_ssh(&parsed_spec.fetch_spec) {
                    hosted.committish = Some(commit.clone());
                    let tarball = hosted.tarball();

                    if let Some(tarball) = tarball {
                        resolution = Some(Resolution::TarballResolution {
                            integrity: None,
                            registry: None,
                            tarball,
                        });
                    }
                }
            }

            if resolution.is_none() {
                resolution = Some(Resolution::GitRepositoryResolution {
                    r#type: String::from("git"),
                    repo: parsed_spec.fetch_spec.clone(),
                    commit: commit.clone(),
                })
            }

            let id = RE_1.replace(&parsed_spec.fetch_spec, "");
            let id = RE_2.replace(&id, "+");
            let id = RE_3.replace(&id, "");

            Some(ResolveResult {
                id: format!("{}/{}", id, commit),
                normalized_pref: parsed_spec.normalized_pref,
                resolution: resolution.unwrap(),
                resolved_via: ResolvedVia::GitRepository,
                latest: None,
                manifest: None,
            })
        }
        None => None,
    }
}

fn is_ssh(git_spec: &str) -> bool {
    git_spec.starts_with("git+ssh://") || git_spec.starts_with("git@")
}

fn get_repo_refs(repo: &str, pref: Option<&str>) -> Result<HashMap<String, String>> {
    dbg!(&repo, &pref);
    let mut git_args = vec![repo];
    if !matches!(pref, Some("HEAD")) {
        git_args.insert(0, "--refs")
    }
    if let Some(pref) = pref {
        git_args.push(pref);
    }

    dbg!(&git_args, &repo);
    let result = Command::new("git")
        .arg("ls-remote")
        .args(git_args)
        .output()?;

    let refs = std::str::from_utf8(&result.stdout)?
        .par_split('\n')
        .filter_map(|line| {
            let mut iter = line.split('\t');
            let commit = iter.next();
            let ref_name = iter.next();

            match (commit, ref_name) {
                (Some(commit), Some(ref_name)) => Some((ref_name.to_string(), commit.to_string())),
                _ => None,
            }
        })
        .collect::<HashMap<_, _>>();

    Ok(refs)
}

fn resolve_ref(repo: &str, pref: &str, git_range: Option<&str>) -> Result<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[0-9a-f]{7,40}$").unwrap();
    }

    if RE.is_match(pref) {
        Ok(pref.to_string())
    } else {
        dbg!(&git_range.as_ref(), &pref);
        let refs = get_repo_refs(
            repo,
            match git_range {
                Some(_) => None,
                None => Some(pref),
            },
        )?;

        dbg!(&refs);
        resolve_ref_from_refs(refs, repo, pref, git_range)
    }
}

fn resolve_ref_from_refs(
    refs: HashMap<String, String>,
    repo: &str,
    pref: &str,
    range: Option<&str>,
) -> Result<String> {
    lazy_static! {
        static ref RE_1: Regex =
            Regex::new(r"^refs/tags/v?(\d+\.\d+\.\d+(?:[-+].+)?)(\^\{\})?$").unwrap();
        static ref RE_2: Regex = Regex::new(r"^refs/tags/").unwrap();
        static ref RE_3: Regex = Regex::new(r"\^\{\}$").unwrap();
    }

    match range {
        Some(range) => {
            let range = VersionReq::parse(range).unwrap();
            let ref_v_tag: Option<String> = refs
                .par_iter()
                .map(|(k, _)| k)
                .filter(|key| RE_1.is_match(key))
                .map(|key| {
                    let key = RE_2.replace(key, "");
                    let key = RE_3.replace(&key, "").to_string();
                    key
                })
                .filter_map(|key| {
                    Version::parse(&key)
                        .map_or(None, |version| range.matches(&version).then(|| version))
                })
                .max_by(|version: &Version, other: &Version| version.cmp(other))
                .map(|version: Version| version.to_string());
            let commit_id = match ref_v_tag.as_ref().and_then(|ref_v_tag| {
                refs.get(&format!("refs/tags/{}^{{}}", ref_v_tag))
                    .or_else(|| refs.get(&format!("refs/tags/{}", ref_v_tag)))
            }) {
                Some(tag) => tag,
                None => bail!(
                    "Could not resolve {} to a commit of {}. Available versions are {}",
                    range,
                    repo,
                    ""
                ),
            };

            Ok(commit_id.to_string())
        }
        None => {
            let commit_id = refs
                .get(pref)
                .or_else(|| refs.get(format!("refs/tags/{}^{{}}", pref).as_str()))
                .or_else(|| refs.get(format!("refs/tags/{}", pref).as_str()))
                .or_else(|| refs.get(format!("refs/heads/{}", pref).as_str()));

            match commit_id {
                Some(commit_id) => Ok(commit_id.to_string()),
                None => bail!("Could not resolve {} to a commit of {}.", pref, repo),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::panic;

    #[test]
    fn with_commit() {
        let resolve_result =
            resolve_from_git("zkochan/is-negative#163360a8d3ae6bee9524541043197ff356f8ed99");

        assert_eq!(resolve_result, Some(ResolveResult {
            id: "github.com/zkochan/is-negative/163360a8d3ae6bee9524541043197ff356f8ed99".to_string(),
            normalized_pref: "github:zkochan/is-negative#163360a8d3ae6bee9524541043197ff356f8ed99".to_string(),
            latest: None,
            manifest: None,
            resolution: Resolution::TarballResolution {
                integrity: None,
                registry: None,
                tarball: "https://codeload.github.com/zkochan/is-negative/tar.gz/163360a8d3ae6bee9524541043197ff356f8ed99".to_string(),
            },
            resolved_via: ResolvedVia::GitRepository,
        }))
    }

    #[test]
    fn with_no_commit() {
        for _ in 0..2 {
            let resolve_result = resolve_from_git("zkochan/is-negative");
            assert_eq!(resolve_result, Some(ResolveResult {
                latest: None,
                manifest: None,
                id: "github.com/zkochan/is-negative/1d7e288222b53a0cab90a331f1865220ec29560c".to_string(),
                normalized_pref: "github:zkochan/is-negative".to_string(),
                resolution: Resolution::TarballResolution{
                    integrity: None,
                    registry: None,
                    tarball: "https://codeload.github.com/zkochan/is-negative/tar.gz/1d7e288222b53a0cab90a331f1865220ec29560c".to_string(),
                },
                resolved_via: ResolvedVia::GitRepository,
            }));
        }
    }

    #[test]
    fn with_no_commit_when_main_branch_is_not_master() {
        let resolve_result = resolve_from_git("zoli-forks/cmd-shim");

        assert_eq!(resolve_result, Some(ResolveResult {
            id: "github.com/zoli-forks/cmd-shim/a00a83a1593edb6e395d3ce41f2ef70edf7e2cf5".to_string(),
            normalized_pref: "github:zoli-forks/cmd-shim".to_string(),
            resolution: Resolution::TarballResolution{
                integrity: None,
                registry: None,
                tarball: "https://codeload.github.com/zoli-forks/cmd-shim/tar.gz/a00a83a1593edb6e395d3ce41f2ef70edf7e2cf5".to_string(),
            },
            latest: None,
            manifest: None,
            resolved_via: ResolvedVia::GitRepository,
        }))
    }

    #[test]
    fn with_partial_commit() {
        let resolve_result = resolve_from_git("zoli-forks/cmd-shim#a00a83a");

        assert_eq!(
            resolve_result,
            Some(ResolveResult {
                id: "github.com/zoli-forks/cmd-shim/a00a83a".to_string(),
                normalized_pref: "github:zoli-forks/cmd-shim#a00a83a".to_string(),
                resolution: Resolution::TarballResolution {
                    integrity: None,
                    registry: None,
                    tarball: "https://codeload.github.com/zoli-forks/cmd-shim/tar.gz/a00a83a"
                        .to_string(),
                },
                latest: None,
                manifest: None,
                resolved_via: ResolvedVia::GitRepository,
            })
        )
    }

    #[test]
    fn with_branch() {
        let resolve_result = resolve_from_git("zkochan/is-negative#canary");

        assert_eq!(resolve_result, Some(ResolveResult {
            id: "github.com/zkochan/is-negative/4c39fbc124cd4944ee51cb082ad49320fab58121".to_string(),
            normalized_pref: "github:zkochan/is-negative#canary".to_string(),
            resolution: Resolution::TarballResolution{
                integrity: None,
                registry: None,
                tarball: "https://codeload.github.com/zkochan/is-negative/tar.gz/4c39fbc124cd4944ee51cb082ad49320fab58121".to_string(),
            },
            latest: None,
            manifest: None,
            resolved_via: ResolvedVia::GitRepository,
        }))
    }

    #[test]
    fn with_tag() {
        let resolve_result = resolve_from_git("zkochan/is-negative#2.0.1");

        assert_eq!(resolve_result, Some(ResolveResult {
            id: "github.com/zkochan/is-negative/2fa0531ab04e300a24ef4fd7fb3a280eccb7ccc5".to_string(),
            normalized_pref: "github:zkochan/is-negative#2.0.1".to_string(),
            resolution: Resolution::TarballResolution {
                integrity: None,
                registry: None,
                tarball: "https://codeload.github.com/zkochan/is-negative/tar.gz/2fa0531ab04e300a24ef4fd7fb3a280eccb7ccc5".to_string(),
            },
            latest: None,
            manifest: None,
            resolved_via: ResolvedVia::GitRepository,
        }))
    }

    #[test]
    fn with_v_prefixed_tag() {
        let resolve_result = resolve_from_git("andreineculau/npm-publish-git#v0.0.7");

        assert_eq!(resolve_result, Some(ResolveResult {
            id: "github.com/andreineculau/npm-publish-git/a2f8d94562884e9529cb12c0818312ac87ab7f0b".to_string(),
            normalized_pref: "github:andreineculau/npm-publish-git#v0.0.7".to_string(),
            resolution: Resolution::TarballResolution {
                integrity: None,
                registry: None,
                tarball: "https://codeload.github.com/andreineculau/npm-publish-git/tar.gz/a2f8d94562884e9529cb12c0818312ac87ab7f0b".to_string(),
            },
            latest: None,
            manifest: None,
            resolved_via: ResolvedVia::GitRepository,
        }))
    }

    #[test]
    fn with_strict_semver() {
        let resolve_result = resolve_from_git("zkochan/is-negative#semver:1.0.0");

        assert_eq!(resolve_result, Some(ResolveResult {
            id: "github.com/zkochan/is-negative/163360a8d3ae6bee9524541043197ff356f8ed99".to_string(),
            normalized_pref: "github:zkochan/is-negative#semver:1.0.0".to_string(),
            resolution: Resolution::TarballResolution {
                integrity: None,
                registry: None,
                tarball: "https://codeload.github.com/zkochan/is-negative/tar.gz/163360a8d3ae6bee9524541043197ff356f8ed99".to_string(),
            },
            latest: None,
            manifest: None,
            resolved_via: ResolvedVia::GitRepository,
        }))
    }

    #[test]
    #[ignore]
    fn with_strict_semver_v_prefixed() {
        let resolve_result = resolve_from_git("andreineculau/npm-publish-git#semver:v0.0.7");

        assert_eq!(resolve_result, Some(ResolveResult {
            id: "github.com/andreineculau/npm-publish-git/a2f8d94562884e9529cb12c0818312ac87ab7f0b".to_string(),
            normalized_pref: "github:andreineculau/npm-publish-git#semver:v0.0.7".to_string(),
            resolution: Resolution::TarballResolution {
                integrity: None,
                registry: None,
                tarball: "https://codeload.github.com/andreineculau/npm-publish-git/tar.gz/a2f8d94562884e9529cb12c0818312ac87ab7f0b".to_string(),
            },
            latest: None,
            manifest: None,
            resolved_via: ResolvedVia::GitRepository,
        }))
    }

    #[test]
    fn with_range_semver() {
        let resolve_result = resolve_from_git("zkochan/is-negative#semver:^1.0.0");
        assert_eq!(resolve_result, Some(ResolveResult {
            id: "github.com/zkochan/is-negative/9a89df745b2ec20ae7445d3d9853ceaeef5b0b72".to_string(),
            normalized_pref: "github:zkochan/is-negative#semver:^1.0.0".to_string(),
            resolution: Resolution::TarballResolution {
                integrity: None,
                registry: None,
                tarball: "https://codeload.github.com/zkochan/is-negative/tar.gz/9a89df745b2ec20ae7445d3d9853ceaeef5b0b72".to_string(),
            },
            latest: None,
            manifest: None,
            resolved_via: ResolvedVia::GitRepository,
        }));
    }

    #[test]
    #[ignore]
    fn with_range_semver_v_prefixed_tag() {
        let resolve_result = resolve_from_git("andreineculau/npm-publish-git#semver:<=v0.0.7");
        assert_eq!(resolve_result, Some(ResolveResult {
            id: "github.com/andreineculau/npm-publish-git/a2f8d94562884e9529cb12c0818312ac87ab7f0b".to_string(),
            normalized_pref: "github:andreineculau/npm-publish-git#semver:<=v0.0.7".to_string(),
            resolution: Resolution::TarballResolution {
                integrity: None,
                registry: None,
                tarball: "https://codeload.github.com/andreineculau/npm-publish-git/tar.gz/a2f8d94562884e9529cb12c0818312ac87ab7f0b".to_string(),
            },
            latest: None,
            manifest: None,
            resolved_via: ResolvedVia::GitRepository,
        }));
    }

    #[test]
    #[should_panic(
        expected = "Could not resolve bad-ref to a commit of git://github.com/zkochan/is-negative.git."
    )]
    fn fails_when_ref_not_found() {
        // supress panic output to shell
        panic::set_hook(Box::new(|_| {}));
        resolve_from_git("zkochan/is-negative#bad-ref");
    }

    #[test]
    #[should_panic(
        expected = "Could not resolve ^100.0.0 to a commit of git://github.com/zkochan/is-negative.git. Available versions are "
    )]
    fn fails_when_semver_ref_not_found() {
        // supress panic output to shell
        panic::set_hook(Box::new(|_| {}));
        resolve_from_git("zkochan/is-negative#semver:^100.0.0");
    }

    // TODO: make it pass on Windows
    #[test]
    #[cfg(not(windows))]
    fn with_commit_from_non_github_repo() {
        let current_dir = std::env::current_dir().unwrap();
        let local_path = current_dir.to_string_lossy();
        let pref = format!(
            "git+file://{}#988c61e11dc8d9ca0b5580cb15291951812549dc",
            local_path
        );
        let resolve_result = resolve_from_git(pref.as_str());

        assert_eq!(
            resolve_result,
            Some(ResolveResult {
                id: format!("{}/988c61e11dc8d9ca0b5580cb15291951812549dc", local_path),
                normalized_pref: pref,
                resolution: Resolution::GitRepositoryResolution {
                    commit: "988c61e11dc8d9ca0b5580cb15291951812549dc".to_string(),
                    repo: format!("file://{}", local_path),
                    r#type: "git".to_string(),
                },
                latest: None,
                manifest: None,
                resolved_via: ResolvedVia::GitRepository,
            })
        );
    }
}
