use crate::read::LockfileFile;
use crate::types::Lockfile;
use anyhow::Result;
use rayon::prelude::*;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

fn write_lockfile(
    lockfile_filename: &str,
    pkg_path: &str,
    wanted_lockfile: &Lockfile,
    force_shared_format: Option<bool>,
) -> Result<()> {
    let lockfile_path = Path::new(pkg_path).join(lockfile_filename);

    if is_empty_lockfile(wanted_lockfile) {
        std::fs::remove_dir_all(lockfile_path)?;
    } else {
        let yaml_doc = yaml_serialize(wanted_lockfile)?;
        let mut file = NamedTempFile::new()?.persist(lockfile_path)?;
        // file.write(yaml_doc.as_bytes())?;
        write!(file, "{}", yaml_doc)?;
    }

    Ok(())
}

fn is_empty_lockfile(lockfile: &Lockfile) -> bool {
    lockfile
        .importers
        .par_iter()
        .map(|(_, v)| v)
        .all(|importer| {
            importer.specifiers.is_empty()
                && importer
                    .dependencies
                    .as_ref()
                    .map_or(true, |dep| dep.is_empty())
        })
}

fn yaml_serialize(lockfile: &Lockfile) -> serde_yaml::Result<String> {
    let normalized_lockfile = normalize_lockfile(lockfile);
    // the node version sorts the keys of the lockfile here
    // this isn't required with rust/serde since serde respects the order
    // of the keys in the struct definition

    serde_yaml::to_string(&normalized_lockfile)
}

fn normalize_lockfile(lockfile: &Lockfile) -> LockfileFile {
    todo!()
}
