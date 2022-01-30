use anyhow::Result;
use read_file::{read_json5_file, ParsedFile};
use std::{fs, io::Write, path::Path};
use strip_bom::*;
use tempfile::NamedTempFile;
use types::BaseManifest;

pub struct ProjectManifest {
    file_name: String,
    manifest: Option<BaseManifest>,
    writer_options: WriterOptions,
}

pub struct WriterOptions {
    indent: Option<String>,
    insert_final_newline: Option<bool>,
    manifest_path: String,
}

pub fn write_project_manifest(
    file_path: &str,
    manifest: &BaseManifest,
    opts: &WriterOptions,
) -> Result<()> {
    let path = Path::new(file_path);
    let ext = path
        .extension()
        .map(|os_str| os_str.to_string_lossy().to_string())
        .unwrap_or(file_path.to_string());

    if ext == "yaml" {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(&*parent.to_string_lossy())?;
            }
        }

        fs::write(file_path, serde_yaml::to_string(manifest)?)?;
    }

    fs::create_dir_all(
        path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(".".to_string()),
    )?;

    let trailing_new_line = if opts.insert_final_newline.unwrap_or(true) {
        ""
    } else {
        "\n"
    };

    let json = if ext == "json5" {
        json5::to_string(&manifest)?
    } else {
        serde_json::to_string(&manifest)?
    };

    let mut file = NamedTempFile::new()?.persist(file_path)?;
    write!(file, "{}{}", json, trailing_new_line)?;

    Ok(())
}

impl ProjectManifest {
    pub fn write_project_manifest(
        &self,
        updated_manifest: BaseManifest,
        force: bool,
    ) -> Result<Option<()>> {
        // let updated_manifest = &updated_manifest;

        if force
            || self
                .manifest
                .as_ref()
                .map(|manifest| manifest != &updated_manifest)
                .unwrap_or(false)
        {
            Ok(Some(write_project_manifest(
                &self.writer_options.manifest_path,
                &updated_manifest,
                &self.writer_options,
            )?))
        } else {
            Ok(None)
        }
    }
}

pub fn read_project_manifest(project_dir: &str) -> Result<ProjectManifest> {
    let manifest_path = Path::new(project_dir)
        .join("package.json")
        .to_string_lossy()
        .to_string();
    let ParsedFile { data, text } = read_json5_file(&manifest_path)?;

    Ok(ProjectManifest {
        file_name: "package.json".to_string(),
        manifest: Some(data),
        writer_options: WriterOptions {
            manifest_path,
            ..detect_file_formatting(&text)
        },
    })
}

fn detect_file_formatting(text: &str) -> WriterOptions {
    todo!()
}

pub fn read_project_manifest_only(project_dir: &str) {}

mod read_file {
    use anyhow::{anyhow, Result};
    use std::fs;
    use strip_bom::StripBom;
    use types::BaseManifest;

    pub struct ParsedFile {
        pub data: BaseManifest,
        pub text: String,
    }

    pub fn read_json5_file(file_path: &str) -> Result<ParsedFile> {
        let text = read_file_without_bom(file_path)?;
        let data = json5::from_str::<BaseManifest>(&text)
            .map_err(|err| anyhow!("{} in {}", err.to_string(), file_path))?;

        Ok(ParsedFile { data, text })
    }

    fn read_file_without_bom(path: &str) -> Result<String> {
        Ok(fs::read_to_string(path)?.strip_bom().to_string())
    }
}
