mod git_host;
use std::{
    collections::{HashMap, HashSet},
    ops::Not,
};
mod git_host_info;
use git_host::GitHost;
use git_host_info::{BY_DOMAIN, BY_SHORTCUT, GIT_HOSTS};
use lazy_static::lazy_static;
use lru::LruCache;
use regex::Regex;
use std::sync::Mutex;
use url::Url;
use urlencoding::{decode, encode};

lazy_static! {
    static ref CACHE: Mutex<LruCache<String, Option<GitHost>>> = Mutex::new(LruCache::new(1000));
    static ref KNOWN_PROTOCOLS: [&'static str; 11] = [
        "github:",
        "bitbucket:",
        "gitlab:",
        "gist:",
        "sourcehut:",
        "http:",
        "https:",
        "git:",
        "git+ssh:",
        "git+https:",
        "ssh:"
    ];
    static ref PROTOCOL_TO_REPRESENTATION: HashMap<&'static str, &'static str> =
        HashMap::from_iter([
            ("git+ssh:", "sshurl",),
            ("git+https:", "https",),
            ("ssh:", "sshurl",),
            ("git:", "git"),
        ]);
    static ref AUTH_PROTOCOLS: HashSet<&'static str> =
        HashSet::from_iter(["git:", "https:", "git+https:", "http:", "git+http:"]);
}

fn protocol_to_representation(protocol: &str) -> &str {
    PROTOCOL_TO_REPRESENTATION
        .get(protocol)
        .copied()
        .unwrap_or_else(|| &protocol[..protocol.len() - 1])
}

pub fn from_url(git_url: &str) -> Option<GitHost> {
    let key = git_url.to_string();
    let mut cache = CACHE.lock().unwrap();

    if !cache.contains(&key) {
        let result = from_url_internal(git_url);
        cache.put(key.clone(), result);
    }

    cache.get(&key).unwrap().clone()
}

fn from_url_internal(git_url: &str) -> Option<GitHost> {
    let url = if is_github_shorthand(git_url) {
        format!("github:{}", git_url)
    } else {
        correct_protocol(git_url)
    };
    let parsed = parse_git_url(&url);
    let protocol = format!("{}:", parsed.scheme());
    let git_host_shortcut = BY_SHORTCUT.get(&protocol);
    let git_host_domain = BY_DOMAIN.get({
        let host = parsed.domain();
        host.and_then(|host| host.strip_prefix("www."))
            .unwrap_or_else(|| host.unwrap_or(""))
    });
    let git_host_name = git_host_shortcut.or(git_host_domain);

    match git_host_name {
        Some(git_host_name) => {
            let git_host_info = GIT_HOSTS.get(git_host_name).expect("githost not found");
            let auth = AUTH_PROTOCOLS.contains(protocol.as_str()).then(|| {
                match (parsed.username(), parsed.password()) {
                    (username, Some(password)) => format!("{}:{}", username, password),
                    (username, None) => username.to_string(),
                }
            });

            let mut committish = None;
            let mut user = None;
            let mut project = String::new();
            let default_representation;

            match git_host_shortcut {
                Some(_) => {
                    let mut path_name = if parsed.path().starts_with('/') {
                        &parsed.path()[1..]
                    } else {
                        parsed.path()
                    };
                    let first_at = path_name.find('@');

                    if let Some(first_at) = first_at {
                        path_name = &path_name[first_at + 1..];
                    }

                    // js compat
                    // let first_at = first_at.map(|i| i as isize).unwrap_or(-1);
                    let last_slash = path_name.rfind('/');

                    match last_slash {
                        Some(last_slash) => {
                            user = Some(decode(&path_name[0..last_slash]).unwrap());

                            // if user.unwrap().is_empty() {
                            //     user = None;
                            // }

                            project = decode(&path_name[last_slash + 1..]).unwrap().to_string();
                        }
                        None => {
                            if let Ok(data) = decode(path_name) {
                                project = data.to_string();
                            }
                        }
                    }

                    if let Some(project_without_suffix) = project.strip_suffix(".git") {
                        project = String::from(project_without_suffix);
                    }

                    if let Some(hash) = parsed.fragment() {
                        committish = Some(decode(&hash).unwrap());
                    }

                    default_representation = "shortcut";
                }
                None => {
                    if !git_host_info.protocols.contains(&protocol.as_str()) {
                        return None;
                    }

                    let segments = git_host_info.extract(&parsed);
                    match segments {
                        Some(segments) => {
                            user = segments
                                .user
                                .as_ref()
                                .map(|segment| decode(segment).unwrap().into_owned().into());
                            project = decode(segments.project).unwrap().to_string();
                            committish = segments
                                .committish
                                .map(|committish| decode(committish).unwrap());
                            default_representation = protocol_to_representation(&protocol);
                        }
                        None => return None,
                    }
                }
            }

            Some(GitHost {
                r#type: git_host_name.to_string(),
                user: user.map(|c| c.to_string()),
                auth,
                project,
                committish: committish.map(|c| c.to_string()),
                default_representation: default_representation.into(), // opts
            })
        }
        None => None,
    }
}

// try to parse the url as its given to us, if that throws
// then we try to clean the url and parse that result instead
// THIS FUNCTION SHOULD NEVER THROW
fn parse_git_url(git_url: &str) -> Url {
    dbg!(git_url);
    let result = Url::parse(git_url);

    if let Ok(result) = result {
        return result;
    }

    let corrected_url = correct_url(git_url);
    dbg!(&corrected_url);
    Url::parse(&corrected_url).unwrap()
}

fn correct_url(git_url: &str) -> String {
    let first_at = git_url.find('@').map(|i| i as isize).unwrap_or(-1);
    let last_hash = git_url.rfind('#');
    let mut first_colon = git_url.find(':').map(|i| i as isize).unwrap_or(-1);
    let mut last_colon = if let Some(last_hash) = last_hash {
        (&git_url[last_hash..])
            .rfind(':')
            .map(|i| i as isize)
            .unwrap()
    } else {
        git_url.rfind(':').map(|i| i as isize).unwrap()
    };

    let mut corrected = String::new();
    if last_colon > first_at {
        // the last : comes after the first @ (or there is no @)
        // like it would in:
        // proto://hostname.com:user/repo
        // username@hostname.com:user/repo
        // :password@hostname.com:user/repo
        // username:password@hostname.com:user/repo
        // proto://username@hostname.com:user/repo
        // proto://:password@hostname.com:user/repo
        // proto://username:password@hostname.com:user/repo
        // then we replace the last : with a / to create a valid path
        corrected = format!(
            "{}/{}",
            &git_url[0..last_colon as usize],
            &git_url[last_colon as usize + 1..]
        );
        // // and we find our new : positions
        first_colon = corrected.find(':').map(|i| i as isize).unwrap_or(-1);
        last_colon = corrected.rfind(':').map(|i| i as isize).unwrap_or(-1);
    }

    if first_colon == -1 && !git_url.contains("//") {
        // we have no : at all
        // as it would be in:
        // username@hostname.com/user/repo
        // then we prepend a protocol
        corrected = format!("git+ssh://{}", corrected);
    }

    corrected
}

fn correct_protocol(url: &str) -> String {
    let first_colon = url.find(':').map(|i| i as isize).unwrap_or(-1);
    let proto = &url[0..first_colon as usize + 1];

    if KNOWN_PROTOCOLS.contains(&proto) {
        return url.to_string();
    }

    let first_at = url.find('@').map(|i| i as isize).unwrap_or(-1);
    if first_at > -1 {
        if first_at > first_colon {
            return format!("git+ssh://{}", url);
        } else {
            return url.to_string();
        }
    }

    let double_slash = url.find("//").map(|i| i as isize).unwrap_or(-1);
    if double_slash == first_colon + 1 {
        return url.to_string();
    }

    format!("{}//{}", proto, &url[first_colon as usize + 1..])
}

// make this more performant
// why is this doing so much index lookups?
// https://github.com/npm/hosted-git-info/blob/main/index.js
fn is_github_shorthand(url: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\s").unwrap();
    }

    let first_hash = url.find('#').map(|i| i as isize).unwrap_or(-1);
    let first_slash = url.find('/').map(|i| i as isize).unwrap_or(-1);
    let second_slash = (&url[first_slash as usize + 1..])
        .find('/')
        .map(|i| i as isize)
        .unwrap_or(-1);
    let first_colon = url.find(':').map(|i| i as isize).unwrap_or(-1);
    // let firstSpace = /\s/.exec(arg);
    let first_space = RE.find(url);
    let first_at = url.find('@').map(|i| i as isize).unwrap_or(-1);

    let space_only_after_hash = match first_space {
        Some(first_space) => first_hash > -1 && first_space.start() > first_hash as usize,
        None => true,
    };
    let at_only_after_hash = first_at == -1 || (first_hash > -1 && first_at > first_hash);
    let colon_only_after_hash = first_colon == -1 || (first_hash > -1 && first_colon > first_hash);
    let second_slash_only_after_hash =
        second_slash == -1 || (first_hash > -1 && second_slash > first_hash);
    let has_slash = first_slash > 0;
    // if a # is found, what we really want to know is that the character immediately before # is not a /
    let does_not_end_with_slash = if first_hash > -1 {
        let index = first_hash - 1;
        let char = if index < 0 {
            None
        } else {
            url.chars().nth(index as usize)
        };
        matches!(char, Some('/')).not()
    } else {
        url.ends_with('/').not()
    };
    let does_not_start_with_dot = url.starts_with('.').not();

    space_only_after_hash
        && has_slash
        && does_not_end_with_slash
        && does_not_start_with_dot
        && at_only_after_hash
        && colon_only_after_hash
        && second_slash_only_after_hash
}

#[cfg(test)]
mod tests {}
