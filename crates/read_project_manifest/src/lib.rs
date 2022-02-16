use anyhow::Result;
use read_file::{read_json_file, ParsedFile};
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;
use types::BaseManifest;

#[derive(Debug, PartialEq)]
pub struct WriterOptions {
    pub insert_final_newline: Option<bool>,
    pub manifest_path: String,
}

#[derive(Debug)]
pub struct ProjectManifest {
    pub file_name: String,
    pub manifest: Option<BaseManifest>,
    pub writer_options: WriterOptions,
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
        .unwrap_or_else(|| file_path.to_string());

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
            .unwrap_or_else(|| ".".to_string()),
    )?;

    let trailing_new_line = if opts.insert_final_newline.unwrap_or(true) {
        ""
    } else {
        "\n"
    };

    let json = if ext == "json5" {
        json5::to_string(&manifest)?
    } else {
        serde_json::to_string_pretty(&manifest)?
    };

    let mut file = NamedTempFile::new()?.persist(file_path)?;
    write!(file, "{}{}", json, trailing_new_line)?;

    Ok(())
}

impl ProjectManifest {
    pub fn write_project_manifest(
        &self,
        updated_manifest: &BaseManifest,
        force: bool,
    ) -> Result<Option<()>> {
        // let updated_manifest = &updated_manifest;

        if force || self.manifest.as_ref().expect("manifest not found") != updated_manifest {
            Ok(Some(write_project_manifest(
                &self.writer_options.manifest_path,
                updated_manifest,
                &self.writer_options,
            )?))
        } else {
            Ok(None)
        }
    }
}

pub fn read_project_manifest_only(project_dir: &str) -> Result<BaseManifest> {
    let ProjectManifest { manifest, .. } = read_project_manifest(project_dir)?;
    Ok(manifest.expect("manifest not found"))
}

pub fn read_project_manifest<P: AsRef<Path>>(project_dir: P) -> Result<ProjectManifest> {
    let manifest_path = project_dir.as_ref().join("package.json");
    let manifest_path_string = manifest_path.to_string_lossy().to_string();

    // maybe support other package.[ext] formats like yaml in the future"
    match read_json_file(&manifest_path_string) {
        Ok(ParsedFile { data, text }) => Ok(ProjectManifest {
            file_name: "package.json".to_string(),
            manifest: Some(data),
            writer_options: WriterOptions {
                manifest_path: manifest_path_string,
                insert_final_newline: Some(text.ends_with('\n')),
            },
        }),
        Err(error) => match error.downcast_ref::<std::io::Error>() {
            Some(file_error) => match dbg!(file_error.kind()) {
                std::io::ErrorKind::NotFound => Ok(ProjectManifest {
                    file_name: "package.json".to_string(),
                    manifest: None,
                    // rewrite this struct to take a closure for writer rather than implementing method
                    writer_options: WriterOptions {
                        manifest_path: manifest_path_string,
                        insert_final_newline: None,
                    },
                }),
                _ => Err(error),
            },
            None => Err(error),
        },
    }
}

pub fn read_exact_project_manifest<P: AsRef<Path>>(manifest_path: P) -> Result<ProjectManifest> {
    let manifest_path = manifest_path.as_ref().to_string_lossy();
    let manifest = read_json_file(&manifest_path)?;

    Ok(ProjectManifest {
        file_name: "package.json".to_string(),
        manifest: Some(manifest.data),
        writer_options: WriterOptions {
            insert_final_newline: None,
            manifest_path: manifest_path.to_string(),
        },
    })
}

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

    pub fn read_json_file(file_path: &str) -> Result<ParsedFile> {
        let text = read_file_without_bom(file_path)?;
        let data = serde_json::from_str::<BaseManifest>(&text)
            .map_err(|err| anyhow!("{} in {}", err.to_string(), file_path))?;

        Ok(ParsedFile { data, text })
    }

    fn read_file_without_bom(path: &str) -> Result<String> {
        Ok(fs::read_to_string(path)?.strip_bom().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use lazy_static::lazy_static;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tempfile::tempdir;

    lazy_static! {
        // ensures that tests run synchronously
        static ref RESOURCE: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn read_manifest_file() {
        let _shared = RESOURCE.lock().unwrap();

        assert_matches!(
            read_project_manifest(Path::new("fixtures").join("package-json")),
            Ok(ProjectManifest {
                manifest: Some(BaseManifest {
                    name: Some(name),
                    version: Some(version),
                    ..
                }),
                ..
            }) => {
                assert_eq!(name, "foo");
                assert_eq!(version, "1.0.0");
            }
        );
    }

    #[test]
    fn empty_manifest_if_no_package_json() {
        let _shared = RESOURCE.lock().unwrap();

        assert_matches!(
            read_project_manifest("fixtures"),
            Ok(ProjectManifest { manifest: None, .. })
        )
    }

    #[test]
    fn do_not_save_manifest_if_not_changed() {
        let _shared = RESOURCE.lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();

        assert!(std::env::set_current_dir(&temp_dir).is_ok());
        std::fs::write("package.json", "{\n\"name\": \"foo\"\n}\n").unwrap();

        let manifest = read_project_manifest(std::env::current_dir().unwrap()).unwrap();

        let updated_manifest = BaseManifest {
            dependencies: Some(HashMap::from_iter([("bar".into(), "1.0.0".into())])),
            ..manifest.manifest.clone().unwrap()
        };
        manifest
            .write_project_manifest(&updated_manifest, false)
            .unwrap();

        let raw_manifest = fs::read_to_string("package.json").unwrap();
        assert_eq!(
            raw_manifest,
            "{\n  \"name\": \"foo\",\n  \"dependencies\": {\n    \"bar\": \"1.0.0\"\n  }\n}"
        );

        assert!(std::env::set_current_dir(original_dir).is_ok())
    }

    #[test]
    fn fail_on_invalid_json() {
        let _shared = RESOURCE.lock().unwrap();

        assert_matches!(read_project_manifest(Path::new("fixtures").join("invalid-package-json")), Err(err) => {
            assert_eq!(err.to_string(), "expected `,` or `}` at line 3 column 3 in fixtures/invalid-package-json/package.json");
        });
    }
}
