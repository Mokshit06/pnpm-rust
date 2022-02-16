use lazy_static::lazy_static;
use regex::Regex;
use urlencoding::encode;

// const BUILTINS: [&str; 64] = [
//     "_http_agent",
//     "_http_client",
//     "_http_common",
//     "_http_incoming",
//     "_http_outgoing",
//     "_http_server",
//     "_stream_duplex",
//     "_stream_passthrough",
//     "_stream_readable",
//     "_stream_transform",
//     "_stream_wrap",
//     "_stream_writable",
//     "_tls_common",
//     "_tls_wrap",
//     "assert",
//     "assert/strict",
//     "async_hooks",
//     "buffer",
//     "child_process",
//     "cluster",
//     "console",
//     "constants",
//     "crypto",
//     "dgram",
//     "diagnostics_channel",
//     "dns",
//     "dns/promises",
//     "domain",
//     "events",
//     "fs",
//     "fs/promises",
//     "http",
//     "http2",
//     "https",
//     "inspector",
//     "module",
//     "net",
//     "os",
//     "path",
//     "path/posix",
//     "path/win32",
//     "perf_hooks",
//     "process",
//     "punycode",
//     "querystring",
//     "readline",
//     "repl",
//     "stream",
//     "stream/promises",
//     "string_decoder",
//     "sys",
//     "timers",
//     "timers/promises",
//     "tls",
//     "trace_events",
//     "tty",
//     "url",
//     "util",
//     "util/types",
//     "v8",
//     "vm",
//     "wasi",
//     "worker_threads",
//     "zlib",
// ];
const BLACKLIST: [&str; 2] = ["node_modules", "favicon.ico"];

lazy_static! {
    static ref SCOPED_PACKAGE_PATTERN: Regex = Regex::new(r"^(?:@([^/]+?)[/])?([^/]+?)$").unwrap();
}

#[derive(Debug, PartialEq)]
pub struct ValidNames {
    pub valid_for_old_packages: bool,
    // valid_for_new_packages: bool,
}

impl ValidNames {
    fn new(valid_for_old_packages: bool) -> Self {
        Self {
            valid_for_old_packages,
        }
    }
}

fn encode_exclamation(data: &str) -> String {
    encode(data).replace("%21", "!")
}

pub fn validate_npm_package_name(name: &str) -> ValidNames {
    if name.is_empty()
        || name.starts_with('.')
        || name.starts_with('_')
        || BLACKLIST.contains(&&*name.to_lowercase())
        || name.trim() != name
    {
        return ValidNames::new(false);
    }

    if encode_exclamation(name) != name {
        let name_captures = SCOPED_PACKAGE_PATTERN.captures(name);
        if let Some(name_captures) = name_captures {
            let user = &name_captures.get(1).map(|user| user.as_str());
            let package = &name_captures.get(2).map(|package| package.as_str());

            if let (Some(user), Some(package)) = (user, package) {
                if encode_exclamation(user) == *user && encode_exclamation(package) == *package {
                    return ValidNames::new(true);
                }
            }
        }

        return ValidNames::new(false);
    }

    ValidNames::new(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! validate {
        ($url:expr, $is_valid:expr) => {
            assert_eq!(
                validate_npm_package_name($url),
                ValidNames {
                    valid_for_old_packages: $is_valid,
                }
            );
        };
    }

    #[test]
    fn validate_npm_package() {
        validate!("some-package", true);
        validate!("example.com", true);
        validate!("under_score", true);
        validate!("period.js", true);
        validate!("123numeric", true);
        validate!("crazy!", true);
        validate!("@npm/thingy", true);
        validate!("@npm-zors/money!time.js", true);

        validate!("", false);
        validate!(".start-with-period", false);
        validate!("_start-with-underscore", false);
        validate!("contain:colons", false);
        validate!(" leading-space", false);
        validate!("trailing-space ", false);
        validate!("s/l/a/s/h/e/s", false);
        validate!("node_modules", false);
        validate!("favicon.ico", false);

        validate!("http", true);

        validate!("ifyouwanttogetthesumoftwonumberswherethosetwonumbersarechosenbyfindingthelargestoftwooutofthreenumbersandsquaringthemwhichismultiplyingthembyitselfthenyoushouldinputthreenumbersintothisfunctionanditwilldothatforyou-", true);
        validate!("ifyouwanttogetthesumoftwonumberswherethosetwonumbersarechosenbyfindingthelargestoftwooutofthreenumbersandsquaringthemwhichismultiplyingthembyitselfthenyoushouldinputthreenumbersintothisfunctionanditwilldothatforyou", true);

        validate!("CAPITAL-LETTERS", true);
    }
}
