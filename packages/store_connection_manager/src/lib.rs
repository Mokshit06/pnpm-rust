mod create_new_store_controller;
mod server_connection_info_dir;

use anyhow::{bail, Result};
use create_new_store_controller::{create_new_store_controller, CreateNewStoreControllerOptions};
use log::{info, warn};
use serde::Deserialize;
use server_connection_info_dir::server_connection_info_dir;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::time::{Duration, Instant};
use std::{fs, thread};
use store_path::store_path;

pub struct CreateStoreControllerOptions {
    dir: String,
    store_dir: Option<String>,
    workspace_dir: Option<String>,
    use_running_store_server: Option<bool>,
    use_store_server: Option<bool>,
}

pub struct StoreController;

pub struct StoreEntry {
    ctrl: StoreController,
    dir: String,
}

impl StoreEntry {}

pub fn create_or_connect_store_controller_cached(
    store_control_cache: &mut HashMap<String, StoreEntry>,
    opts: CreateStoreControllerOptions,
) -> Result<&StoreEntry> {
    let store_dir = store_path(&opts.dir, opts.store_dir.as_deref())?;

    Ok(store_control_cache
        .entry(store_dir.to_string_lossy().to_string())
        .or_insert_with(|| StoreEntry {
            ctrl: StoreController,
            dir: String::from(""),
        }))
}

pub fn create_or_connect_store_controller(
    opts: CreateStoreControllerOptions,
) -> Result<StoreEntry> {
    let store_dir = store_path(
        &opts.workspace_dir.unwrap_or(opts.dir),
        opts.store_dir.as_deref(),
    )?;
    let connection_info_dir = server_connection_info_dir(&store_dir);
    let store_dir = store_dir.to_string_lossy().to_string();
    let server_json_path = connection_info_dir.join("server.json");
    let server_json = try_load_server_json(&server_json_path.to_string_lossy(), false)?;

    if let Some(server_info) = server_json {
        todo!("implement @pnpm/cli-meta");
        if server_info.pnpm_version != "" {
            warn!("The store server runs on pnpm v{}. It is recommended to connect with the same version (current is v{})", server_info.pnpm_version, "");
        }
        info!("A store server is running. All store manipulations are delegated to it.");

        return Ok(StoreEntry {
            ctrl: StoreController,
            dir: store_dir,
        });
    }

    if opts.use_running_store_server.unwrap_or(false) {
        bail!("NO_STORE_SERVER: No store server is running.")
    }

    if opts.use_store_server.unwrap_or(false) {
        // spawn a daemon thread
        run_server_in_background(&store_dir);
        let server_json = try_load_server_json(&server_json_path.to_string_lossy(), true)?.unwrap();
        info!("A store server has been started. To stop it, use `pnpm server stop`");

        return Ok(StoreEntry {
            ctrl: StoreController,
            dir: store_dir,
        });
    }

    Ok(create_new_store_controller(
        CreateNewStoreControllerOptions {},
        &store_dir,
    ))
}

fn run_server_in_background(store_dir: &str) {
    thread::spawn(|| {});
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionOptions {
    remote_prefix: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerInfo {
    connection_options: ConnectionOptions,
    pid: i32,
    pnpm_version: String,
}

fn try_load_server_json(
    server_json_path: &str,
    should_retry_on_enoent: bool,
) -> Result<Option<ServerInfo>> {
    let mut before_first_attempt = true;
    let instant = Instant::now();

    loop {
        if !before_first_attempt {
            let elapsed = instant.elapsed();
            if elapsed.as_secs() >= 10 {
                return match fs::remove_file(server_json_path) {
                    Ok(_) => Ok(None),
                    Err(err) => match err.kind() {
                        ErrorKind::NotFound => Ok(None),
                        // Either the server.json was manually removed or another process already removed it.
                        _ => return Err(err.into()),
                    },
                };
            }

            std::thread::sleep(Duration::from_secs(2));
        }

        before_first_attempt = false;
        let server_json_str = match fs::read_to_string(server_json_path) {
            Ok(contents) => contents,
            Err(err) => {
                if !matches!(err.kind(), ErrorKind::NotFound) {
                    return Err(err.into());
                }
                if !should_retry_on_enoent {
                    return Ok(None);
                }
                continue;
            }
        };
        let server_json = match serde_json::from_str::<Option<ServerInfo>>(&server_json_str) {
            Ok(server_json) => server_json,
            Err(_) => continue,
        };

        if let Some(server_json) = server_json {
            return Ok(Some(server_json));
        } else {
            // Our server should never write null to server.json, even though it is valid json.
            bail!("server.json was modified by a third party")
        }
    }
}
