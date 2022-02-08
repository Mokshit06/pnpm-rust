#[derive(Debug, PartialEq, Clone)]
pub struct GitHost {
    pub r#type: String,
    pub user: Option<String>,
    pub auth: Option<String>,
    pub project: String,
    pub committish: Option<String>,
    pub default_representation: String,
}

enum TemplateType {
    FileTemplate,
    SSHTemplate,
}

use templates::{file_template, ssh_template, TemplateOpts};

impl GitHost {
    pub fn hash(&self) -> String {
        self.committish
            .as_ref()
            .map(|committish| format!("#{}", committish))
            .unwrap_or_else(|| "".to_string())
    }

    pub fn ssh(&self, opts: TemplateOpts) -> String {
        self.fill(TemplateType::SSHTemplate, opts)
    }

    pub fn file(&self, opts: TemplateOpts) -> String {
        self.fill(TemplateType::FileTemplate, opts)
    }

    fn fill(&self, template_type: TemplateType, mut opts: TemplateOpts) -> String {
        if opts.path.starts_with('/') {
            let slice = &opts.path[1..];
            opts.path = slice.to_string();
        }

        if opts.no_committish {
            opts.committish = None;
        }

        let result = match template_type {
            TemplateType::SSHTemplate => ssh_template(&self.r#type, &opts),
            TemplateType::FileTemplate => file_template(&self.r#type, &opts),
            _ => format!(""),
        };

        if opts.no_git_plus && result.starts_with("git+") {
            String::from(&result[4..])
        } else {
            result
        }
    }
}

mod templates {
    pub struct TemplateOpts {
        pub path: String,
        pub no_committish: bool,
        pub committish: Option<String>,
        pub no_git_plus: bool,
        pub user: String,
        pub project: String,
        pub domain: String,
        pub auth: Option<String>,
    }

    pub fn file_template(git_host_type: &str, opts: &TemplateOpts) -> String {
        match git_host_type {
            "github" => format!(
                "https://{}raw.githubusercontent.com/{}/{}/{}/{}",
                maybe_join(&[opts.auth.as_deref(), Some("@")]),
                opts.user,
                opts.project,
                maybe_encode(opts.committish.as_deref()).unwrap_or_else(|| "master".to_string()),
                opts.path
            ),
            "gist" => format!(
                "https://gist.githubusercontent.com/{}/{}/raw{}/{}",
                opts.user,
                opts.project,
                maybe_join(&[
                    Some("/"),
                    maybe_encode(opts.committish.as_deref()).as_deref()
                ]),
                opts.path
            ),
            "sourcehut" => format!(
                "https://{}/{}/{}/blob/{}/{}",
                opts.domain,
                opts.user,
                opts.project,
                maybe_encode(opts.committish.as_deref()).unwrap_or_else(|| "main".to_string()),
                opts.path
            ),
            _ => format!(
                "https://{}/{}/{}/raw/{}/{}",
                opts.domain,
                opts.user,
                opts.project,
                maybe_encode(opts.committish.as_deref()).unwrap_or_else(|| "master".to_string()),
                opts.path
            ),
        }
    }

    pub fn ssh_template(git_host_type: &str, opts: &TemplateOpts) -> String {
        match git_host_type {
            "gist" => format!(
                "git@{}:{}.git{}",
                opts.domain,
                opts.project,
                maybe_join(&[Some("#"), opts.committish.as_deref()])
            ),
            _ => format!(
                "git@{}:{}/{}.git{}",
                opts.domain,
                opts.user,
                opts.project,
                maybe_join(&[Some("#"), opts.committish.as_deref()])
            ),
        }
    }

    fn maybe_encode(arg: Option<&str>) -> Option<String> {
        arg.map(|arg| urlencoding::encode(arg).to_string())
    }

    fn maybe_join(args: &[Option<&str>]) -> String {
        if args.iter().all(|arg| arg.is_some()) {
            args.iter()
                .map(|arg| arg.unwrap())
                .collect::<Vec<_>>()
                .join(",")
        } else {
            String::from("")
        }
    }
}
