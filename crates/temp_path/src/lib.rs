use std::path::{Path, PathBuf};
use std::process;

use rand::{distributions::Alphanumeric, Rng};

fn unique_id() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>()
}

pub fn temp_path_atomic<P: AsRef<Path>>(folder: P) -> PathBuf {
    Path::new(folder.as_ref()).join(format!("_tmp_{}_{}", process::id(), unique_id()))
}

pub fn temp_path<P: AsRef<Path>>(folder: P) -> PathBuf {
    Path::new(folder.as_ref()).join(format!("_tmp_{}", unique_id()))
}
