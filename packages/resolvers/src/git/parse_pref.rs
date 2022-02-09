use hosted_git_info::{self, GitHost, TemplateOpts};
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use std::process::Command;
use url::Url;
use urlencoding::decode;

pub struct Hosted {
    r#type: String,
    user: String,
    project: String,
    committish: String,
    tarball: Option<Box<dyn Fn()>>,
}

impl Hosted {
    fn tarball(&self) -> Option<String> {
        todo!()
    }
}

pub struct HostedPackageSpec {
    fetch_spec: String,
    hosted: Option<Hosted>,
    normalized_pref: String,
    git_committish: Option<String>,
    git_range: Option<String>,
}

const GIT_PROTOCOLS: [&str; 8] = [
    "git",
    "git+http",
    "git+https",
    "git+rsync",
    "git+ftp",
    "git+file",
    "git+ssh",
    "ssh",
];

pub fn parse_pref(pref: &str) -> Option<HostedPackageSpec> {
    let hosted = hosted_git_info::from_url(pref);

    match hosted {
        Some(hosted) => Some(from_hosted_git(hosted)),
        None => match pref.find(':') {
            Some(colons_pos) => {
                let protocol = &pref[0..colons_pos];
                if GIT_PROTOCOLS.contains(&protocol.to_lowercase().as_str()) {
                    let parsed_url = Url::parse(&escape_colon(pref));

                    match parsed_url {
                        Ok(parsed_url) if parsed_url.scheme().is_empty() => None,
                        Ok(parsed_url) => {
                            if parsed_url.scheme() == "git+ssh" {
                                let matches = match_git_scp(pref);

                                if let Some((fetch_spec, git_committish)) = matches {
                                    return Some(HostedPackageSpec {
                                        fetch_spec: fetch_spec.unwrap_or("").to_string(),
                                        git_committish: git_committish
                                            .map(|committish| committish.to_string()),
                                        git_range: None,
                                        hosted: None,
                                        normalized_pref: pref.to_string(),
                                    });
                                }
                            }

                            let committish = match parsed_url.fragment() {
                                Some(hash) if hash.len() > 1 => Some(decode(hash).unwrap()),
                                Some(_) | None => None,
                            };

                            let (git_range, git_committish) =
                                set_git_committish(committish.as_deref());
                            Some(HostedPackageSpec {
                                normalized_pref: pref.to_string(),
                                git_committish,
                                git_range,
                                hosted: None,
                                fetch_spec: url_to_fetch_spec(parsed_url),
                            })
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
            None => None,
        },
    }
}

fn url_to_fetch_spec(mut url: Url) -> String {
    url.set_fragment(None);
    let fetch_spec = url.as_str();
    if let Some(fetch_spec) = fetch_spec.strip_prefix("git+") {
        fetch_spec.to_string()
    } else {
        fetch_spec.to_string()
    }
}

fn match_git_scp(spec: &str) -> Option<(Option<&str>, Option<&str>)> {
    lazy_static! {
        static ref RE_1: Regex =
            RegexBuilder::new(r"^git\+ssh://([^:]+:[^#]+(?:\.git)?)(?:#(.*))$")
                .case_insensitive(true)
                .build()
                .unwrap();
        static ref RE_2: Regex = RegexBuilder::new(r":[0-9]+/?.*$")
            .case_insensitive(true)
            .build()
            .unwrap();
    }

    RE_1.captures(spec).and_then(|matched| {
        let matches = match matched.get(1) {
            Some(first_match) => RE_2.is_match(first_match.as_str()),
            None => false,
        };

        if matches {
            return None;
        }

        Some((
            matched.get(1).map(|m| m.as_str()),
            matched.get(2).map(|m| m.as_str()),
        ))
    })
}

fn escape_colon(url: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r":([^/\d]|\d+[^:/\d])").unwrap();
    }

    if !url.contains('@') {
        return url.to_string();
    }
    let mut iter = url.split('@');
    let front = iter.next().map(|front| front.to_string());
    let escaped_backs = iter.map(|e| RE.replace(e, ":/$1").to_string());

    if let Some(front) = front {
        std::iter::once(front)
            .chain(escaped_backs)
            .collect::<Vec<_>>()
            .join("@")
    } else {
        escaped_backs.collect::<Vec<_>>().join("@")
    }
}

fn from_hosted_git(hosted: GitHost) -> HostedPackageSpec {
    let mut fetch_spec = None;
    let git_url = hosted
        .git(TemplateOpts {
            no_committish: true,
            ..Default::default()
        })
        .or_else(|| {
            hosted.ssh(TemplateOpts {
                no_committish: true,
                ..Default::default()
            })
        });

    if let Some(git_url) = git_url {
        if access_repository(&git_url) {
            fetch_spec = Some(git_url);
        }
    }

    if fetch_spec.is_none() {
        let https_url = hosted.https(TemplateOpts {
            no_git_plus: true,
            no_committish: true,
            ..Default::default()
        });

        if let Some(https_url) = https_url {
            if hosted.auth.is_some() && access_repository(&https_url) {
                let (git_range, git_committish) = set_git_committish(hosted.committish.as_deref());

                return HostedPackageSpec {
                    normalized_pref: format!("git+{}", https_url),
                    fetch_spec: https_url,
                    hosted: Some(Hosted {
                        tarball: None,
                        committish: hosted.committish.unwrap(),
                        project: hosted.project,
                        r#type: hosted.r#type,
                        user: hosted.user.unwrap(),
                    }),
                    git_range,
                    git_committish,
                };
            } else {
                todo!("implement @pnpm/fetch and use it here")
            }
        }
    }

    if fetch_spec.is_none() {
        fetch_spec = hosted.ssh(TemplateOpts {
            no_committish: true,
            ..Default::default()
        });
    }

    let (git_range, git_committish) = set_git_committish(hosted.committish.as_deref());
    let normalized_pref = hosted.shortcut(TemplateOpts::default());

    HostedPackageSpec {
        fetch_spec: fetch_spec.expect("fetch_spec not found"),
        hosted: Some(Hosted {
            tarball: None,
            committish: hosted.committish.unwrap(),
            project: hosted.project,
            r#type: hosted.r#type,
            user: hosted.user.unwrap(),
        }),
        normalized_pref: normalized_pref.unwrap(),
        git_range,
        git_committish,
    }
}

fn set_git_committish(committish: Option<&str>) -> (Option<String>, Option<String>) {
    match committish {
        Some(committish) if committish.len() >= 7 && &committish[0..7] == "semver:" => {
            (None, Some(String::from(&committish[7..])))
        }
        Some(_) | None => (None, None),
    }
}

fn access_repository(repository: &str) -> bool {
    Command::new("git")
        .args(["ls-remote", "--exit-code", repository, "HEAD"])
        .output()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_colon_is_escaped() {
        let tests = [
            [
                "ssh://username:password@example.com:repo.git",
                "ssh://username:password@example.com/repo.git",
            ],
            [
                "ssh://username:password@example.com:repo/@foo.git",
                "ssh://username:password@example.com/repo/@foo.git",
            ],
            [
                "ssh://username:password@example.com:22/repo/@foo.git",
                "ssh://username:password@example.com:22/repo/@foo.git",
            ],
            [
                "ssh://username:password@example.com:22repo/@foo.git",
                "ssh://username:password@example.com/22repo/@foo.git",
            ],
            [
                "git+ssh://username:password@example.com:repo.git",
                "ssh://username:password@example.com/repo.git",
            ],
            [
                "git+ssh://username:password@example.com:repo/@foo.git",
                "ssh://username:password@example.com/repo/@foo.git",
            ],
            [
                "git+ssh://username:password@example.com:22/repo/@foo.git",
                "ssh://username:password@example.com:22/repo/@foo.git",
            ],
        ];

        for [input, expected] in tests {
            let parsed = parse_pref(input).unwrap();
            assert_eq!(parsed.fetch_spec, expected, "error in {}", input);
        }
    }
}
