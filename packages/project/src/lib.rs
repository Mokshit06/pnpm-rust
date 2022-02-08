use anyhow::Result;
use read_project_manifest::{write_project_manifest, WriterOptions};
use std::collections::HashMap;
use types::BaseManifest;

// pub trait WriteProject {
//     fn write_project_manifest(
//         &self,
//         updated_manifest: &BaseManifest,
//         force: bool,
//     ) -> Result<Option<()>>;
// }

#[derive(Debug, PartialEq)]
pub struct Project {
    pub dir: String,
    pub manifest: BaseManifest,
    pub writer_options: WriterOptions,
}

impl Project {
    fn write_project_manifest(
        &self,
        updated_manifest: &BaseManifest,
        force: bool,
    ) -> Result<Option<()>> {
        // let updated_manifest = &updated_manifest;

        if force || &self.manifest != updated_manifest {
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

pub struct Graph<'a> {
    pub dependencies: Vec<String>,
    pub package: &'a Project,
}

pub type ProjectsGraph<'a> = HashMap<String, Graph<'a>>;
