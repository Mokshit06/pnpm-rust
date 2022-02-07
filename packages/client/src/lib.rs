use credentials_by_uri::get_credentials_by_uri;
use std::collections::HashMap;

mod credentials_by_uri;
mod nerf_dart;

pub struct ClientOptions {
    auth_config: HashMap<String, String>,
    retry: Option<()>,
    timeout: Option<i64>,
    user_agent: Option<String>,
    user_config: Option<HashMap<String, String>>,
}

pub fn create_client(opts: ClientOptions) {}

// pub fn create_resolver(opts: &ClientOptions) {
//     let fetch_from_registry = create_fetch_from_registry(opts);
//     // memoize function
//     let get_credentials =
//         Box::new(|registry: &str| get_credentials_by_uri(&opts.auth_config, registry, None));

//     create_resolve(fetch_from_registry, get_credentials, opts);
// }

// pub fn create_fetchers(
//     fetch_from_registry: FetchFromRegistry,
//     get_credentials: impl GetCredentials,
//     retry: Option<()>,
// ) {
// }
