use crate::comver_to_semver;
use crate::git_merge_file::autofix_merge_conflicts;
use crate::types::{Lockfile, ProjectSnapshot};
use semver::Version;
use std::path::Path;

pub struct ReadCurrentLockFileOpts {
    wanted_version: Option<usize>,
    ignore_incompatible: bool,
}

// TODO: change to async function
pub fn read_current_lockfile(
    virtual_store_dir: &str,
    opts: ReadCurrentLockFileOpts,
) -> Result<ReadResult, ReadError> {
    let lockfile_path = Path::new(virtual_store_dir).join("lock.yaml");

    read(
        lockfile_path.to_str().unwrap(),
        virtual_store_dir,
        ReadOptions {
            autofix_merge_conflicts: Some(false),
            wanted_version: opts.wanted_version,
            ignore_incompatible: opts.ignore_incompatible,
        },
    )
}

// remove BOM character from string
fn strip_bom(string: &str) -> &str {
    if string.starts_with("\u{feff}") {
        &string[3..]
    } else {
        &string[..]
    }
}

pub struct ReadResult {
    lockfile: Option<Lockfile>,
    had_conflicts: bool,
}

struct LockfileFile {
    lockfile: Lockfile,
    snapshot: ProjectSnapshot,
}

pub enum ReadError {
    NotFound(std::io::Error),
    YamlParse(serde_yaml::Error),
    BreakingChange(String),
}

#[derive(Default)]
struct ReadOptions {
    autofix_merge_conflicts: Option<bool>,
    wanted_version: Option<usize>,
    ignore_incompatible: bool,
}

fn read(lockfile_path: &str, _prefix: &str, opts: ReadOptions) -> Result<ReadResult, ReadError> {
    let lockfile_raw_content = match std::fs::read_to_string(lockfile_path) {
        Ok(content) => String::from(strip_bom(&content)),
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

    if opts.wanted_version.is_none()
        || Version::parse(&lockfile_semver).unwrap().major
            == Version::parse(&comver_to_semver(&opts.wanted_version.unwrap().to_string()))
                .unwrap()
                .major
    {
        Ok(ReadResult {
            lockfile: Some(lockfile),
            had_conflicts,
        })
    } else if opts.ignore_incompatible {
        Ok(ReadResult {
            lockfile: None,
            had_conflicts: false,
        })
    } else {
        Err(ReadError::BreakingChange(String::from(lockfile_path)))
    }
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
                ReadError::BreakingChange(path) => panic!("Breaking change: {}", path),
            },
        };

        // todo: use snapshot tests to assert that the parsed file is correct
        println!("{:#?}", result.lockfile.unwrap());
    }
}
