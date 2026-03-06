use directories::ProjectDirs;
use serde::Deserialize;
use std::{env, path::PathBuf, sync::LazyLock};

/// Project Name: Filaments
pub static PROJECT_NAME: LazyLock<String> =
    LazyLock::new(|| env!("CARGO_CRATE_NAME").to_uppercase());

/// The OS-agnostic data directory for the project.
pub static DATA_DIRECTORY: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    env::var(format!("{}_DATA", PROJECT_NAME.clone()))
        .ok()
        .map(PathBuf::from)
});

/// The OS-agnostic config directory for the project.
pub static CONFIG_DIRECTORY: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
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
    DATA_DIRECTORY.clone().unwrap_or_else(|| {
        project_directory().map_or_else(
            || PathBuf::from(".").join(".data"),
            |proj_dirs| proj_dirs.data_local_dir().to_path_buf(),
        )
    })
}

/// Returns the path to the OS-agnostic config directory.
#[expect(dead_code)]
pub fn get_config_dir() -> PathBuf {
    CONFIG_DIRECTORY.clone().unwrap_or_else(|| {
        project_directory().map_or_else(
            || PathBuf::from(".").join(".config"),
            |proj_dirs| proj_dirs.config_local_dir().to_path_buf(),
        )
    })
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "suri-codes", env!("CARGO_PKG_NAME"))
}
