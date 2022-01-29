mod git_merge_file;
mod merge_changes;
pub mod read;
pub mod satisfies_package_manifest;
pub mod types;
mod write;

fn comver_to_semver(comver: &str) -> String {
    if !comver.contains(".") {
        format!("{}.0.0", comver)
    } else {
        format!("{}.0", comver)
    }
}
