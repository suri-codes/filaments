use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{env, path::PathBuf, sync::LazyLock};

// #[expect(dead_code)]
static PROJECT_NAME: LazyLock<String> = LazyLock::new(|| env!("CARGO_CRATE_NAME").to_uppercase());

#[expect(dead_code)]
static DATA_FOLDER: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    env::var(format!("{}_DATA", PROJECT_NAME.clone()))
        .ok()
        .map(PathBuf::from)
});
#[expect(dead_code)]
static CONFIG_FOLDER: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    env::var(format!("{}_CONFIG", PROJECT_NAME.clone()))
        .ok()
        .map(PathBuf::from)
});

/// The App Config and Data locations.
#[derive(Clone, Debug, Deserialize, Default)]
#[expect(dead_code)]
pub struct AppDirs {
    #[serde(default)]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub config_dir: PathBuf,
}

/// Configuration for the App
#[expect(dead_code)]
pub struct Config {
    pub app_dirs: AppDirs, // pub data_dir: PathBuf,
                           // pub keybindings: KeyBindings,

                           // pub styles: Styles,
}

#[expect(dead_code)]
impl Config {
    pub fn new() -> Self {
        todo!()
    }
}

/// Returns the path to the OS-agnostic data directory.
pub fn get_data_dir() -> PathBuf {
    let directory = if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    };
    directory
}

/// Returns the path to the OS-agnostic config directory.
pub fn get_config_dir() -> PathBuf {
    let directory = if let Some(s) = CONFIG_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    };
    directory
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "suri-codes", env!("CARGO_PKG_NAME"))
}
