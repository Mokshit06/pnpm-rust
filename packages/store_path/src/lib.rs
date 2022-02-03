use anyhow::{bail, Result};
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};
use temp_path::temp_path;

mod temp_path;

const STORE_VERSION: &str = "v3";

pub fn store_path() {}

fn store_path_relative_to_home(pkg_root: &str, rel_store: &str) -> Result<PathBuf> {
    let temp_file = temp_path(pkg_root);
    fs::create_dir_all(temp_file.parent().unwrap_or_else(|| Path::new(".")))?;
    File::create(&temp_file)?;
    let home_dir = dirs::home_dir().expect("home_dir not found");

    if can_link_to_subdir(&temp_file, &home_dir)? {
        fs::remove_file(temp_file)?;

        return Ok(home_dir.join(rel_store).join(STORE_VERSION));
    }

    let result = match root_link_target(&temp_file) {
        Ok(mountpoint) => {
            let mountpoint_parent = mountpoint.parent().expect("Expected parent of mountpoint");

            todo!("implement the rest from [https://github.com/pnpm/store-path/blob/main/src/index.ts]")
        }
        Err(_) => home_dir.join(rel_store).join(STORE_VERSION),
    };

    fs::remove_file(temp_file)?;
    Ok(result)
}

fn can_link_to_subdir(file_to_link: &PathBuf, dir: &PathBuf) -> Result<bool> {
    let temp_dir = temp_path(dir);
    let result = match fs::create_dir_all(&temp_dir) {
        Ok(_) => can_link(&file_to_link, temp_path(dir)),
        Err(_) => false,
    };
    fs::remove_dir_all(temp_dir)?;

    Ok(result)
}

fn root_link_target<P: AsRef<Path>>(file_path: P) -> Result<PathBuf> {
    let file_path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(file_path);
    let end = file_path.parent().unwrap_or_else(|| Path::new("."));
    let mut components = end.components();
    let mut dir = if let Some(Component::RootDir) = components.next() {
        Path::new("/").to_path_buf()
    } else {
        Path::new("").to_path_buf()
    };

    loop {
        if can_link(&file_path, temp_path(&dir)) {
            break Ok(dir);
        } else if dir == end {
            bail!("{} cannot be linked anywhere", file_path.to_string_lossy())
        } else {
            dir = next_path(&dir.to_string_lossy(), end);
        }
    }
}

fn next_path(from: &str, to: &Path) -> PathBuf {
    use relative_path::RelativePath;

    let diff = RelativePath::new(&from).to_path(to);
    let diff = diff.to_string_lossy();
    // convert `usize` to `isize` for js compat
    let sep_index = diff
        .chars()
        .position(|c| c == std::path::MAIN_SEPARATOR)
        .map(|c| c as isize)
        .unwrap_or(-1);
    let next = if sep_index > 0 {
        // since sep_index > 0, it can safely be converted back to `usize`
        &diff[0..sep_index as usize]
    } else {
        &diff[..]
    };

    Path::new(from).join(next)
}

fn can_link<P: AsRef<Path>, S: AsRef<Path>>(temp_dir: P, file_to_link: S) -> bool {
    let new_path = temp_path(&temp_dir);
    match fs::hard_link(file_to_link, &new_path) {
        Ok(_) => {
            // `ok()` will ignore this error
            fs::remove_file(new_path).ok();
            true
        }
        Err(_) => false,
    }
}

fn safe_rmdir() {}

fn dirs_are_equal() {}

fn get_homedir() {}

fn is_homepath() {}
