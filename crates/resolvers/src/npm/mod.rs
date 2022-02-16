pub struct ResolverFactoryOptions {
    cache_dir: String,
    full_metadata: Option<bool>,
    offline: Option<bool>,
    prefer_offline: Option<bool>,
    retry: Option<RetryTimeoutOptions>,
    timeout: Option<i64>,
}

pub struct NpmResolver;

impl NpmResolver {
    pub fn new<F>(registry: Registry, get_credentials: F, opts: ResolverFactoryOptions) -> Self
    where
        F: Fn() -> Credentials,
    {
        // figure a way to memoize this

        Self
    }
}
