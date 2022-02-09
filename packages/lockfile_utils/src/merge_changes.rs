use itertools::Itertools;
use std::collections::HashMap;

use crate::comver_to_semver;
use crate::read::LockfileFile;
use crate::types::{Lockfile, ProjectSnapshot};
use semver::Version;

// implement https://github.dev/pnpm/pnpm/blob/334e5340a45d0812750a2bad8fdda5266401c362/packages/merge-lockfile-changes/src/index.ts#L29
pub fn merge_changes(ours: LockfileFile, theirs: LockfileFile) -> LockfileFile {
    let theirs_semver =
        Version::parse(comver_to_semver(theirs.lockfile_version.as_str()).as_str()).unwrap();
    let ours_semver =
        Version::parse(comver_to_semver(ours.lockfile_version.as_str()).as_str()).unwrap();

    let mut new_lockfile = LockfileFile::new(if theirs_semver > ours_semver {
        theirs.lockfile_version.clone()
    } else {
        ours.lockfile_version.clone()
    });

    for importer_id in ours
        .importers
        .iter()
        .map(|(k, _)| k)
        .chain(theirs.importers.iter().map(|(k, _)| k))
        .unique()
    {
        new_lockfile
            .importers
            .insert(importer_id.clone(), ProjectSnapshot::new());

        // new_lockfile.importers.
    }

    todo!();

    new_lockfile
}
