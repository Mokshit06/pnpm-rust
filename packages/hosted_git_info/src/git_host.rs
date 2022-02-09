#[derive(Debug, PartialEq, Clone)]
pub struct GitHost {
    pub r#type: String,
    pub user: Option<String>,
    pub auth: Option<String>,
    pub project: String,
    pub committish: Option<String>,
    pub default_representation: String,
}

pub use templates::TemplateOpts;
use templates::{
    file_template, git_template, https_template, shortcut_template, ssh_template, TemplateType,
};

impl GitHost {
    pub fn hash(&self) -> String {
        self.committish
            .as_ref()
            .map(|committish| format!("#{}", committish))
            .unwrap_or_else(|| "".to_string())
    }

    pub fn ssh(&self, opts: TemplateOpts) -> Option<String> {
        self.fill(TemplateType::SSHTemplate, opts.merge_with_host(&self))
    }

    pub fn file(&self, opts: TemplateOpts) -> Option<String> {
        self.fill(TemplateType::FileTemplate, opts.merge_with_host(&self))
    }

    pub fn git(&self, opts: TemplateOpts) -> Option<String> {
        self.fill(TemplateType::GitTemplate, opts.merge_with_host(&self))
    }

    pub fn shortcut(&self, opts: TemplateOpts) -> Option<String> {
        self.fill(TemplateType::ShortcutTemplate, opts.merge_with_host(&self))
    }

    pub fn https(&self, opts: TemplateOpts) -> Option<String> {
        self.fill(TemplateType::HTTPSType, opts.merge_with_host(&self))
    }

    fn fill(&self, template_type: TemplateType, mut opts: TemplateOpts) -> Option<String> {
        if opts.path.starts_with('/') {
            let slice = &opts.path[1..];
            opts.path = slice.to_string();
        }

        if opts.no_committish {
            opts.committish = None;
        }

        let result = match template_type {
            TemplateType::SSHTemplate => Some(ssh_template(&self.r#type, &opts)),
            TemplateType::FileTemplate => Some(file_template(&self.r#type, &opts)),
            TemplateType::GitTemplate => git_template(&self.r#type, &opts),
            TemplateType::HTTPSType => Some(https_template(&self.r#type, &opts)),
            TemplateType::ShortcutTemplate => Some(shortcut_template(&self.r#type, &opts)),
        };

        match result {
            Some(result) => {
                if opts.no_git_plus && result.starts_with("git+") {
                    Some(String::from(&result[4..]))
                } else {
                    Some(result)
                }
            }
            None => None,
        }
    }
}

mod templates {
    use crate::GitHost;

    pub enum TemplateType {
        FileTemplate,
        SSHTemplate,
        GitTemplate,
        HTTPSType,
        ShortcutTemplate,
    }

    #[derive(Default)]
    pub struct TemplateOpts {
        pub path: String,
        pub no_committish: bool,
        pub committish: Option<String>,
        pub no_git_plus: bool,
        pub user: Option<String>,
        pub project: Option<String>,
        pub domain: String,
        pub auth: Option<String>,
    }

    impl TemplateOpts {
        pub fn merge_with_host(self, host: &GitHost) -> Self {
            TemplateOpts {
                auth: self.auth.or_else(|| host.auth.clone()),
                user: self.user.or_else(|| host.user.clone()),
                project: Some(self.project.unwrap_or_else(|| host.project.clone())),
                path: self.path,
                no_git_plus: self.no_git_plus,
                no_committish: self.no_committish,
                domain: self.domain,
                committish: self.committish.or_else(|| host.committish.clone()),
            }
        }
    }

    pub fn shortcut_template(git_host_type: &str, opts: &TemplateOpts) -> String {
        match git_host_type {
            "gist" => format!(
                "{}:{}{}",
                git_host_type,
                opts.project.as_ref().unwrap(),
                maybe_join(&[Some("#"), opts.committish.as_deref()])
            ),
            _ => format!(
                "{}:{}/{}{}",
                git_host_type,
                opts.user.as_ref().unwrap(),
                opts.project.as_ref().unwrap(),
                maybe_join(&[Some("#"), opts.committish.as_deref()])
            ),
        }
    }

    pub fn file_template(git_host_type: &str, opts: &TemplateOpts) -> String {
        match git_host_type {
            "github" => format!(
                "https://{}raw.githubusercontent.com/{}/{}/{}/{}",
                maybe_join(&[opts.auth.as_deref(), Some("@")]),
                opts.user.as_ref().unwrap(),
                opts.project.as_ref().unwrap(),
                maybe_encode(opts.committish.as_deref()).unwrap_or_else(|| "master".to_string()),
                opts.path
            ),
            "gist" => format!(
                "https://gist.githubusercontent.com/{}/{}/raw{}/{}",
                opts.user.as_ref().unwrap(),
                opts.project.as_ref().unwrap(),
                maybe_join(&[
                    Some("/"),
                    maybe_encode(opts.committish.as_deref()).as_deref()
                ]),
                opts.path
            ),
            "sourcehut" => format!(
                "https://{}/{}/{}/blob/{}/{}",
                opts.domain,
                opts.user.as_ref().unwrap(),
                opts.project.as_ref().unwrap(),
                maybe_encode(opts.committish.as_deref()).unwrap_or_else(|| "main".to_string()),
                opts.path
            ),
            _ => format!(
                "https://{}/{}/{}/raw/{}/{}",
                opts.domain,
                opts.user.as_ref().unwrap(),
                opts.project.as_ref().unwrap(),
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
                opts.project.as_ref().unwrap(),
                maybe_join(&[Some("#"), opts.committish.as_deref()])
            ),
            _ => format!(
                "git@{}:{}/{}.git{}",
                opts.domain,
                opts.user.as_ref().unwrap(),
                opts.project.as_ref().unwrap(),
                maybe_join(&[Some("#"), opts.committish.as_deref()])
            ),
        }
    }

    pub fn git_template(git_host_type: &str, opts: &TemplateOpts) -> Option<String> {
        match git_host_type {
            "github" => Some(format!(
                "git://{}{}/{}/{}.git{}",
                maybe_join(&[opts.auth.as_deref(), Some("@")]),
                opts.domain,
                opts.user.as_ref().unwrap(),
                opts.project.as_ref().unwrap(),
                maybe_join(&[Some("#"), opts.committish.as_deref()])
            )),
            "gist" => Some(format!(
                "git://{}/{}.git{}",
                opts.domain,
                opts.project.as_ref().unwrap(),
                maybe_join(&[Some("#"), opts.committish.as_deref()]),
            )),
            _ => None,
        }
    }

    pub fn https_template(git_host_type: &str, opts: &TemplateOpts) -> String {
        match git_host_type {
            "gitlab" => format!(
                "git+https://{}{}/{}/{}.git{}",
                maybe_join(&[opts.auth.as_deref(), Some("@")]),
                opts.domain,
                opts.user.as_ref().unwrap(),
                opts.project.as_ref().unwrap(),
                maybe_join(&[Some("#"), opts.committish.as_deref()])
            ),
            "gist" => format!(
                "git+https://{}/{}.git{}",
                opts.domain,
                opts.project.as_ref().unwrap(),
                maybe_join(&[Some("#"), opts.committish.as_deref()])
            ),
            "sourcehut" => format!(
                "https://{}/{}/{}.git{}",
                opts.domain,
                opts.user.as_ref().unwrap(),
                opts.project.as_ref().unwrap(),
                maybe_join(&[Some("#"), opts.committish.as_deref()])
            ),
            _ => format!(
                "git+https://{}{}/{}/{}.git{}",
                maybe_join(&[opts.auth.as_deref(), Some("@")]),
                opts.domain,
                opts.user.as_ref().unwrap(),
                opts.project.as_ref().unwrap(),
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
