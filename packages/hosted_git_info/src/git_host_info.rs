use lazy_static::lazy_static;
use std::{collections::HashMap, ops::Not};
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

#[derive(Debug)]
pub struct GitHostSegments<'a> {
    pub user: Option<&'a str>,
    pub project: &'a str,
    pub committish: Option<&'a str>,
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
                    let mut iter = url.as_str().strip_prefix(&format!("{}:",url.scheme())).unwrap().split('/');
                    iter.next();
                    let user = iter.next();
                    let mut project = iter.next();
                    let r#type = iter.next();
                    let mut committish = iter.next();

                    dbg!(&user,&project,&r#type,&committish);

                    // if type is not `tree`
                    if r#type.is_some() && r#type.unwrap() != "tree" {
                        return None;
                    }

                    if r#type.is_none() {
                        committish = Some(&url.fragment().unwrap()[1..]);
                    }

                    if let Some(ref mut project) = project {
                        if let Some(project_without_suffix) = project.strip_suffix(".git") {
                            *project = project_without_suffix;
                        }
                    }

                    if user.is_none() || project.is_none() {
                        return None;
                    }

                    Some(GitHostSegments {
                        committish,
                        project: project.unwrap(),
                        user
                    })
                },
            }
        ),
        (
            "bitbucket",
            GitHostInfo {
                protocols: &["git+ssh:", "git+https:", "ssh:", "https:"],
                domain: "bitbucket.org",
                treepath: Some("src"),
                extract: |_| None
            }
        ),
        (
            "gitlab",
            GitHostInfo {
                protocols: &["git+ssh:", "git+https:", "ssh:", "https:"],
                domain: "gitlab.com",
                treepath: Some("tree"),
                extract: |_| None
            }
        ),
        (
            "gist",
            GitHostInfo {
                protocols: &["git:", "git+ssh:", "git+https:", "ssh:", "https:"],
                domain: "gist.github.com",
                treepath: None,
                extract: |_| None
            }
        ),
        (
            "sourcehut",
            GitHostInfo {
                protocols: &["git+ssh:", "https:"],
                domain: "git.sr.ht",
                treepath: Some("tree"),
                extract: |_| None
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

}
