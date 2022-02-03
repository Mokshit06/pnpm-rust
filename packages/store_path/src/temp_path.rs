use std::path::{Path, PathBuf};
use std::process;

use rand::{distributions::Alphanumeric, Rng};

pub fn temp_path<P: AsRef<Path>>(folder: P) -> PathBuf {
    let rand_str = rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>();

    Path::new(folder.as_ref()).join(format!("_tmp_{}_{}", process::id(), rand_str))
}
