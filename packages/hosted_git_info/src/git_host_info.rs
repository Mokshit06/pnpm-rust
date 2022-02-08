use lazy_static::lazy_static;
use std::borrow::Cow;
use std::collections::HashMap;
use url::Url;

pub struct GitHostInfo {
    pub protocols: &'static [&'static str],
    pub domain: &'static str,
    pub treepath: Option<&'static str>,
    pub extract: for<'a> fn(&'a Url) -> Option<GitHostSegments<'a>>,
}

impl GitHostInfo {
    pub fn extract<'a>(&self, url: &'a Url) -> Option<GitHostSegments<'a>> {
        (self.extract)(url)
    }
}

#[derive(Debug, PartialEq)]
pub struct GitHostSegments<'a> {
    pub user: Cow<'a, str>,
    pub project: &'a str,
    pub committish: Option<&'a str>,
}

fn gist_extract(url: &Url) -> Option<GitHostSegments<'_>> {
    let mut segments = url.path_segments().unwrap();
    let mut user = segments.next();
    let mut project = segments.next();
    let aux = segments.next();

    if let Some("raw") = aux {
        return None;
    }

    if project.is_none() {
        if user.is_none() {
            return None;
        }

        project = user.take();
    }

    if let Some(ref mut project) = project {
        if let Some(project_without_suffix) = project.strip_suffix(".git") {
            *project = project_without_suffix;
        }
    }

    todo!()
    // Some(GitHostSegments {
    //     user,
    //     pro
    // })
    // return { user, project, committish: url.hash.slice(1) }
}

lazy_static! {
    pub static ref GIT_HOSTS: HashMap<&'static str, GitHostInfo> = HashMap::from_iter([
        (
            "github",
            GitHostInfo {
                protocols: &["git:", "http:", "git+ssh:", "git+https:", "ssh:", "https:"],
                domain: "github.com",
                treepath: Some("tree"),
                extract: |url| {
                    let mut segments = url.path_segments().unwrap();
                    let user = segments.next();
                    let mut project = segments.next();
                    let r#type = segments.next();
                    let mut committish = segments.next();

                    // if type is not `tree`
                    if matches!(r#type, Some(url_type) if url_type != "tree") {
                        return None;
                    }

                    if r#type.is_none() {
                        committish = url.fragment();
                    }

                    if let Some(ref mut project) = project {
                        if let Some(project_without_suffix) = project.strip_suffix(".git") {
                            *project = project_without_suffix;
                        }
                    }

                    match (user,project) {
                        (Some(user), Some(project)) => Some(GitHostSegments {
                            committish,
                            project,
                            user: Cow::from(user)
                        }),
                        _ => None
                    }
                },
            }
        ),
        (
            "bitbucket",
            GitHostInfo {
                protocols: &["git+ssh:", "git+https:", "ssh:", "https:"],
                domain: "bitbucket.org",
                treepath: Some("src"),
                extract: |url| {
                    let mut segments = url.path_segments().unwrap();
                    let user = segments.next();
                    let mut project = segments.next();
                    let aux = segments.next();

                    if matches!(aux, Some("get")) {
                        return None
                    }

                    if let Some(ref mut project) = project {
                        if let Some(project_without_suffix) = project.strip_suffix(".git") {
                            *project = project_without_suffix;
                        }
                    }

                    match (user,project) {
                        (Some(user), Some(project)) => Some(GitHostSegments {
                            committish: url.fragment(),
                            user: Cow::from(user),
                            project,
                        }),
                        _ => None
                    }
                }
            }
        ),
        (
            "gitlab",
            GitHostInfo {
                protocols: &["git+ssh:", "git+https:", "ssh:", "https:"],
                domain: "gitlab.com",
                treepath: Some("tree"),
                extract: |url: &Url| {
                    let path = url.path();
                    let mut segments = url.path_segments().unwrap().collect::<Vec<_>>();

                    if path.contains("/-/") || path.contains("/archive.tar.gz") {
                        return None;
                    }

                    let mut project = segments.pop();

                    if let Some(ref mut project) = project {
                        if let Some(new_project) = project.strip_prefix(".git") {
                            *project = new_project
                        }
                    }

                    let user = segments.join("/");

                    match (user.is_empty(), project) {
                        (false, Some(project)) => {
                            Some(GitHostSegments {
                                user: Cow::from(user),
                                project,
                                committish: url.fragment()
                            })
                        },
                        _ => None
                    }
                }
            }
        ),
        (
            "gist",
            GitHostInfo {
                protocols: &["git:", "git+ssh:", "git+https:", "ssh:", "https:"],
                domain: "gist.github.com",
                treepath: None,
                extract: gist_extract
            }
        ),
        (
            "sourcehut",
            GitHostInfo {
                protocols: &["git+ssh:", "https:"],
                domain: "git.sr.ht",
                treepath: Some("tree"),
                extract: |_| todo!()
            }
        )
    ]);
    pub static ref BY_SHORTCUT: HashMap<String, &'static str> = GIT_HOSTS
        .iter()
        .map(|(&host, _)| (format!("{}:", host), host))
        .collect();
    pub static ref BY_DOMAIN: HashMap<&'static str, &'static str> = GIT_HOSTS
        .iter()
        .map(|(&host, info)| (info.domain, host))
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn github_extracts_segments_correctly() {
        let host_info = GIT_HOSTS.get("github").unwrap();
        let url = url::Url::parse("git+ssh://git@github.com/npm/hosted-git-info.git").unwrap();
        let segments = host_info.extract(&url);

        assert_eq!(
            segments,
            Some(GitHostSegments {
                committish: None,
                project: "hosted-git-info",
                user: Cow::from("npm")
            })
        );
    }
}
