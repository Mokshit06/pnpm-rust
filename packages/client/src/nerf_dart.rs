use url::{ParseError, Url};

pub trait NerfDart {
    fn to_nerf_dart(&self) -> Result<String, ParseError>;
}

impl NerfDart for str {
    fn to_nerf_dart(&self) -> Result<String, ParseError> {
        let mut url = Url::parse(self)?;
        url.set_query(None);
        url.set_fragment(None);
        url.set_scheme("http").expect("Couldn't set url scheme");
        url.set_password(None).unwrap();
        url.set_username("").unwrap();
        // remove the last segment/path
        url.path_segments_mut().unwrap().pop();

        // remove the `scheme:` from url
        let url = &url.as_str()[5..];

        Ok(if url.ends_with('/') {
            url.to_string()
        } else {
            format!("{}/", url)
        })
    }
}

impl NerfDart for String {
    fn to_nerf_dart(&self) -> Result<String, ParseError> {
        self.as_str().to_nerf_dart()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! valid_nerf_dart {
        ($name: ident, $uri:expr) => {
            valid_nerf_dart!($name, $uri, "//registry.npmjs.org/");
        };
        ($name:ident,$uri:expr, $valid:expr) => {
            #[test]
            fn $name() {
                assert_eq!($uri.to_nerf_dart(), Ok($valid.to_string()));
            }
        };
    }

    valid_nerf_dart!(npm_registry, "http://registry.npmjs.org");
    valid_nerf_dart!(
        npm_registry_with_path,
        "http://registry.npmjs.org/some-package"
    );
    valid_nerf_dart!(
        npm_registry_with_path_and_query,
        "http://registry.npmjs.org/some-package?write=true"
    );
    valid_nerf_dart!(
        npm_registry_with_auth,
        "http://user:pass@registry.npmjs.org/some-package?write=true"
    );
    valid_nerf_dart!(
        npm_registry_with_fragment,
        "http://registry.npmjs.org/#random-hash"
    );
    valid_nerf_dart!(
        npm_registry_with_path_and_fragment,
        "http://registry.npmjs.org/some-package#random-hash"
    );

    valid_nerf_dart!(
        custom_registry,
        "http://relative.couchapp.npm/design/-/rewrite/",
        "//relative.couchapp.npm/design/-/rewrite/"
    );
    valid_nerf_dart!(
        custom_registry_with_port,
        "http://relative.couchapp.npm:8080/design/-/rewrite/",
        "//relative.couchapp.npm:8080/design/-/rewrite/"
    );
    valid_nerf_dart!(
        custom_regsitry_with_port_and_path,
        "http://relative.couchapp.npm:8080/design/-/rewrite/some-package",
        "//relative.couchapp.npm:8080/design/-/rewrite/"
    );
}
