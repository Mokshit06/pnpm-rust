use std::path::{Path, PathBuf};

pub fn server_connection_info_dir(store_path: &Path) -> PathBuf {
    store_path.join("server")
}
