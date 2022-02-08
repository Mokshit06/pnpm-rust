use anyhow::{bail, Result};
use relative_path::RelativePath;
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};
use temp_path::temp_path_atomic;

const STORE_VERSION: &str = "v3";

pub fn store_path(pkg_root: &str, store_path: Option<&str>) -> Result<PathBuf> {
    match store_path {
        Some(store_path) if !is_homepath(store_path) => {
            let store_base_path = path_absolute(store_path, pkg_root);
            let ends_with = format!("{}{}", std::path::MAIN_SEPARATOR, STORE_VERSION);

            if store_base_path.to_string_lossy().ends_with(&ends_with) {
                Ok(store_base_path)
            } else {
                Ok(store_base_path.join(STORE_VERSION))
            }
        }
        Some(_) | None => Ok(store_path_relative_to_home(
            pkg_root,
            store_path.map(|p| &p[2..]).unwrap_or(".pnpm-store"),
        )?),
    }
}

fn path_absolute<P: AsRef<Path>, S: AsRef<Path>>(file_path: P, cwd: S) -> PathBuf {
    let home_dir = dirs::home_dir().expect("home_dir not found");
    let (file_path, cwd) = (file_path.as_ref(), cwd.as_ref());
    let path = file_path.to_string_lossy();

    if is_homepath(&path) {
        home_dir.join(&path[2..])
    } else if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        cwd.join(&file_path)
    }
}

fn store_path_relative_to_home(pkg_root: &str, rel_store: &str) -> Result<PathBuf> {
    let temp_file = temp_path_atomic(pkg_root);
    fs::create_dir_all(temp_file.parent().unwrap_or_else(|| Path::new(".")))?;
    File::create(&temp_file)?;
    let home_dir = dirs::home_dir().expect("home_dir not found");

    if can_link_to_subdir(&temp_file, &home_dir)? {
        fs::remove_file(temp_file)?;

        return Ok(home_dir.join(rel_store).join(STORE_VERSION));
    }

    let result = match root_link_target(&temp_file) {
        Ok(mut mountpoint) => {
            let mountpoint_parent = mountpoint.parent().unwrap_or_else(|| Path::new("."));

            if dirs_are_equal(&mountpoint_parent.to_string_lossy(), &mountpoint)
                && can_link_to_subdir(&temp_file, &mountpoint_parent)?
            {
                mountpoint = mountpoint_parent.to_path_buf();
            }

            if dirs_are_equal(pkg_root, &mountpoint) {
                home_dir.join(rel_store).join(STORE_VERSION)
            } else {
                mountpoint.join(rel_store).join(STORE_VERSION)
            }
        }
        Err(_) => home_dir.join(rel_store).join(STORE_VERSION),
    };

    fs::remove_file(temp_file)?;
    Ok(result)
}

fn dirs_are_equal<S: AsRef<Path>>(dir_1: &str, dir_2: S) -> bool {
    RelativePath::new(dir_1).to_path(dir_2).to_string_lossy() == "."
}

fn can_link_to_subdir<P: AsRef<Path>, S: AsRef<Path>>(file_to_link: P, dir: S) -> Result<bool> {
    let temp_dir = temp_path_atomic(&dir);
    let result = match fs::create_dir_all(&temp_dir) {
        Ok(_) => can_link(&file_to_link, temp_path_atomic(dir)),
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
        if can_link(&file_path, temp_path_atomic(&dir)) {
            break Ok(dir);
        } else if dir == end {
            bail!("{} cannot be linked anywhere", file_path.to_string_lossy())
        } else {
            dir = next_path(&dir.to_string_lossy(), end);
        }
    }
}

fn next_path(from: &str, to: &Path) -> PathBuf {
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
    let new_path = temp_path_atomic(&temp_dir);
    match fs::hard_link(file_to_link, &new_path) {
        Ok(_) => {
            // `ok()` will ignore this error
            fs::remove_file(new_path).ok();
            true
        }
        Err(_) => false,
    }
}

fn is_homepath(file_path: &str) -> bool {
    let mut chars = file_path.chars().map(String::from);
    matches!(chars.position(|c| c == "~/"), Some(0))
        || matches!(chars.position(|c| c == "~\\"), Some(0))
}
