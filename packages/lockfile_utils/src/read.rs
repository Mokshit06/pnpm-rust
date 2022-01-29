use crate::comver_to_semver;
use crate::git_merge_file::autofix_merge_conflicts;
use std::path::Path;
use tokio::fs;

use crate::types::{Lockfile, ProjectSnapshot};

pub struct ReadCurrentLockFileOpts {
    wanted_version: Option<f32>,
    ignore_incompatible: bool,
}

pub async fn read_current_lockfile(virtual_store_dir: &str, opts: ReadCurrentLockFileOpts)
/*-> Lockfile*/
{
    let lockfile_path = Path::new(virtual_store_dir).join("lock.yaml");
}

// remove BOM character from string
fn strip_bom(string: &str) -> &str {
    if string.starts_with("\u{feff}") {
        &string[3..]
    } else {
        &string[..]
    }
}

struct ReadResult {
    lockfile: Option<Lockfile>,
    had_conflicts: bool,
}

struct LockfileFile {
    lockfile: Lockfile,
    snapshot: ProjectSnapshot,
}

enum ReadError {
    NotFound(std::io::Error),
    YamlParse(serde_yaml::Error),
}

#[derive(Default)]
struct ReadOptions {
    autofix_merge_conflicts: Option<bool>,
    wanted_version: Option<usize>,
    ignore_incompatible: bool,
}

fn read(lockfile_path: &str, prefix: &str, opts: ReadOptions) -> Result<ReadResult, ReadError> {
    let lockfile_raw_content = match std::fs::read_to_string(lockfile_path) {
        Ok(str) => str,
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => return Err(ReadError::NotFound(error)),
            _ => {
                return Ok(ReadResult {
                    lockfile: None,
                    had_conflicts: false,
                })
            }
        },
    };
    let mut had_conflicts = false;
    let lockfile = match serde_yaml::from_str::<Lockfile>(&lockfile_raw_content) {
        Ok(lockfile) => lockfile,
        Err(error) => {
            if !opts.autofix_merge_conflicts.unwrap_or(false) {
                return Err(ReadError::YamlParse(error));
            }

            had_conflicts = true;
            autofix_merge_conflicts(&lockfile_raw_content)
        }
    };

    let lockfile_semver = comver_to_semver(&lockfile.lockfile_version);

    Ok(ReadResult {
        lockfile: Some(lockfile),
        had_conflicts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lockfile() {
        let result = match read("./fixtures/test-lock.yaml", "", ReadOptions::default()) {
            Ok(f) => f,
            Err(err) => match err {
                ReadError::NotFound(err) => panic!("{:#?}", err),
                ReadError::YamlParse(err) => panic!("{:#?}", err),
            },
        };

        println!("{:#?}", result.lockfile.unwrap());
    }
}
