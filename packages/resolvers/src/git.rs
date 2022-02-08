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
    use hosted_git_info::{self, GitHost};

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

    // pub fn parse_pref(pref: &str) -> Option<HostedPackageSpec> {
    //     let hosted = hosted_git_info::from_url(pref);
    // }

    // fn from_hosted_git(hosted: &GitHost) -> HostedPackageSpec {
    //     let fetch_spec = None;

    //     let git_url = hosted.git(TemplateOpts { no_comittish: true }).or_else(|| {
    //         hosted.ssh(TemplateOpts {
    //             no_committish: true,
    //         })
    //     });

    //     if let Some(git_url) = git_url {
    //         if access_repository(&git_url) {
    //             fetch_spec = Some(git_url);
    //         }
    //     }

    //     if let None = fetch_spec {}
    // }
}
