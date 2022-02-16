use reqwest::{
    header::{HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT},
    Client,
};

const PNPM_USER_AGENT: &str = "pnpm"; // or maybe make it `${pkg.name}/${pkg.version} (+https://npm.im/${pkg.name})`

const CORGI_DOC: &str = "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";
const JSON_DOC: &str = "application/json";
const MAX_FOLLOWED_REDIRECTS: i32 = 20;

pub struct FetchFromRegistry {
    client: Client,
}

impl FetchFromRegistry {
    pub fn new(full_meta_data: bool, auth: &str) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert(
            ACCEPT,
            (if full_meta_data { JSON_DOC } else { CORGI_DOC })
                .parse()
                .unwrap(),
        );
        headers.insert(AUTHORIZATION, auth.parse().unwrap());
        headers.insert(USER_AGENT, PNPM_USER_AGENT.parse().unwrap());

        Self {
            client: Client::builder().default_headers(headers).build().unwrap(),
        }
    }

    pub fn fetch(&self) {
        // let headers = reqwest::header::HeaderMap
    }
}
