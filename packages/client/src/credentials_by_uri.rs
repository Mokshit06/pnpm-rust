use crate::nerf_dart::*;
use std::process::Command;
use std::{collections::HashMap, path::Path};
use url::ParseError;

#[derive(Default, Debug, PartialEq)]
pub struct Credentials {
    auth_header_value: Option<String>,
    always_auth: Option<bool>,
}

impl Credentials {
    fn new() -> Self {
        Default::default()
    }

    fn merge(self, other: Self) -> Self {
        Self {
            always_auth: self.always_auth.or(other.always_auth),
            auth_header_value: self.auth_header_value.or(other.auth_header_value),
        }
    }
}

pub fn get_credentials_by_uri(
    config: HashMap<String, String>,
    url: &str,
    user_config: Option<HashMap<String, String>>,
) -> Result<Credentials, ParseError> {
    let nerfed = url.to_nerf_dart()?;
    let defnerf = config
        .get("registry")
        .expect("`registry` not found in `config`")
        .to_nerf_dart()?;
    let credentials = get_scoped_credentials(
        &nerfed,
        format!("{}:", nerfed).as_str(),
        &config,
        &user_config,
    );

    if nerfed != defnerf {
        Ok(credentials)
    } else {
        Ok(credentials.merge(get_scoped_credentials(&nerfed, "", &config, &user_config)))
    }
}

fn get_scoped_credentials(
    nerfed: &str,
    scope: &str,
    config: &HashMap<String, String>,
    user_config: &Option<HashMap<String, String>>,
) -> Credentials {
    let mut credentials = Credentials::new();

    if let Some(val) = config.get(&format!("{}always-auth", scope)) {
        credentials.always_auth = Some(val != "false");
    }

    if let Some(user_config) = user_config {
        if let Some(helper) = user_config.get(&format!("{}tokenHelpter", scope)) {
            let path = Path::new(helper);
            if path.is_absolute() || path.exists() {
                // TODO: change to recoverable errors
                panic!("BAD_TOKEN_HELPER_PATH: {}tokenHelper must be an absolute path, without arguments", scope);
            }

            let spawn_result = Command::new("sh").arg("-C").arg(helper).output().unwrap();
            if !matches!(spawn_result.status.code(), Some(0)) {
                let exit_code = spawn_result
                    .status
                    .code()
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| String::from("UNKNOWN"));
                panic!("TOKEN_HELPER_ERROR_STATUS: Error running {} as a token helper, configured as {}tokenHelper. Exit code {}", helper, scope, exit_code);
            }

            credentials.auth_header_value = Some(
                std::str::from_utf8(&spawn_result.stdout)
                    .unwrap()
                    .trim_end()
                    .to_string(),
            );

            return credentials;
        }
    }

    if let Some(auth_token) = config.get(&format!("{}_authToken", scope)) {
        credentials.auth_header_value = Some(format!("Bearer {}", auth_token));
        return credentials;
    }

    if let Some(auth) = config.get(&format!("{}_auth", scope)) {
        credentials.auth_header_value = Some(format!("Basic {}", auth));
        return credentials;
    }

    let username = config.get(&format!("{}username", scope));
    let password = config.get(&format!("{}_password", scope)).map(|password| {
        if scope == "" {
            String::from(password)
        } else {
            String::from_utf8(base64::decode(password).expect("Can't decode password")).unwrap()
        }
    });

    if let (Some(username), Some(password)) = (username, password) {
        credentials.auth_header_value = Some(base64::encode(format!("{}:{}", username, password)));
    };

    credentials
}

macro_rules! hash_map {
    ( $( $key:expr => $value:expr ),+ ) => {{
        let mut map = HashMap::new();

        $(
            map.insert($key, $value);
        )+

        map
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn gets_credentials() {
        let config = hash_map! {
            "registry".into() => "https://registry.npmjs.com".into(),
            "//registry.com/:_authToken".into() => "f23jj93f32dsaf==".into(),
            "//registry.com/:always-auth".into() => "false".into()
        };

        assert_eq!(
            get_credentials_by_uri(config, "https://registry.com", None),
            Ok(Credentials {
                always_auth: Some(false),
                auth_header_value: Some("Bearer f23jj93f32dsaf==".to_string())
            })
        );
    }
}
