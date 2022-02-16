mod create_new_store_controller;
mod server_connection_info_dir;

use anyhow::{bail, Result};
use create_new_store_controller::{create_new_store_controller, CreateNewStoreControllerOptions};
use lazy_static::lazy_static;
use log::{info, warn};
use reqwest::blocking::Client;
use resolvers::base::{PreferredVersions, Resolution, WantedDependency, WorkspacePackages};
use serde::Deserialize;
use server_connection_info_dir::server_connection_info_dir;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::time::{Duration, Instant};
use std::{fs, thread};
use store_path::store_path;

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

pub struct CreateStoreControllerOptions {
    dir: String,
    store_dir: Option<String>,
    workspace_dir: Option<String>,
    use_running_store_server: Option<bool>,
    use_store_server: Option<bool>,
}

pub struct StoreEntry {
    ctrl: StoreController,
    dir: String,
}

impl StoreEntry {}

pub struct StoreController {
    request_package: RequestPackage,
    fetch_package: (),
    import_package: (),
}

pub struct CreateStoreControllerOpts {
    remote_prefix: String,
    concurrency: Option<i32>,
}

impl StoreController {
    pub fn create_or_connect(opts: CreateStoreControllerOptions) -> Result<StoreEntry> {
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
                ctrl: todo!("instantiate store controller"),
                dir: store_dir,
            });
        }

        if opts.use_running_store_server.unwrap_or(false) {
            bail!("NO_STORE_SERVER: No store server is running.")
        }

        if opts.use_store_server.unwrap_or(false) {
            // spawn a daemon thread
            run_server_in_background(&store_dir);
            let server_json =
                try_load_server_json(&server_json_path.to_string_lossy(), true)?.unwrap();
            info!("A store server has been started. To stop it, use `pnpm server stop`");

            return Ok(StoreEntry {
                ctrl: todo!("instantiate store controller"),
                dir: store_dir,
            });
        }

        Ok(create_new_store_controller(
            CreateNewStoreControllerOptions {},
            &store_dir,
        ))
    }

    pub fn create_or_connect_cached(
        store_control_cache: &mut HashMap<String, StoreEntry>,
        opts: CreateStoreControllerOptions,
    ) -> Result<&StoreEntry> {
        let store_dir = store_path(&opts.dir, opts.store_dir.as_deref())?;

        Ok(store_control_cache
            .entry(store_dir.to_string_lossy().to_string())
            .or_insert_with(|| StoreEntry {
                ctrl: todo!("instantiate store controller"),
                dir: String::from(""),
            }))
    }

    pub fn connect(options: CreateStoreControllerOpts) -> Self {
        todo!()
    }

    pub fn close() {}

    pub fn prune() {}

    pub fn upload(built_pkg_location: &str, opts: UploadOptions) {}
}

fn fetch(url: &str, body: HashMap<&str, &str>) -> Result<HashMap<String, String>> {
    lazy_static! {
        #[allow(clippy::non_upper_case_globals)]
        static ref client: Client = Client::new();
    }

    let url = if url.starts_with("https://unix:") {
        url.replace("http://unix:", "unix:")
    } else {
        url.to_string()
    };

    let response = client
        .post(url)
        .json(&body)
        .send()?
        // .await?
        .json::<HashMap<String, String>>()?;
    // .await?;

    if let Some(error) = response.get("error") {
        bail!("{}", error)
    }

    Ok(response)
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

pub struct UploadOptions {
    files_index_file: String,
    engine: String,
}

pub struct RequestPackage {}

impl RequestPackage {
    pub fn request(
        wanted_dependency: WantedDependency,
        optional: Option<bool>,
        options: RequestPackageOptions,
    ) {
    }
}

pub struct RequestPackageOptions<'a> {
    always_try_workspace_packages: Option<bool>,
    current_pkg: Option<Package>,
    default_tag: Option<String>,
    download_priority: i32,
    project_dir: String,
    lockfile_dir: String,
    preferred_versions: PreferredVersions,
    prefer_workspace_packages: Option<bool>,
    registry: String,
    side_effects_cache: Option<bool>,
    skip_fetch: Option<bool>,
    update: Option<bool>,
    workspace_packages: Option<WorkspacePackages<'a>>,
}

struct Package {
    id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    resolution: Option<Resolution>,
}
