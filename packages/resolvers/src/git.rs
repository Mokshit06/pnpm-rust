pub struct GitResolver {
    // wanted_dependency_pref: String,
}

impl GitResolver {
    pub fn new() -> Self {
        Self {}
    }

    pub fn resolve(wanted_dependency_pref: &str) {}
}

mod parse_pref {
    pub struct Hoisted {
        r#type: String,
        user: String,
        project: String,
        committish: String,
        // tarball: () => string | undefined
    }

    impl Hoisted {
        fn tarball(&self) -> Option<String> {
            todo!()
        }
    }

    pub struct HostedPackageSpec {
        fetch_spec: String,
        hosted: Option<Hoisted>,
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
        todo!()
    }
}
