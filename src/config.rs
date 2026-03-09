use directories::ProjectDirs;
use kdl::KdlDocument;
use serde::Deserialize;
use std::{env, path::PathBuf, sync::LazyLock};

use crate::keymap::KeyMap;

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

const DEFAULT_CONFIG: &str = include_str!("../.config/config.kdl");

/// The App Config and Data locations.
#[derive(Clone, Debug, Deserialize, Default)]
#[expect(dead_code)]
pub struct AppConfig {
    #[serde(default)]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub config_dir: PathBuf,
}

/// Configuration for the App
#[expect(dead_code)]
#[derive(Debug, Clone)]
pub struct Config {
    pub app_config: AppConfig,
    pub keymap: KeyMap,
    // pub styles: Styles,
}

impl Config {
    pub fn new() -> Self {
        let default_config: KdlDocument = DEFAULT_CONFIG
            .parse()
            .expect("Default config should always be a valid KDL document.");

        let keymap_node = default_config
            .get("keymap")
            .expect("Config::new Keymap must exist in default config.");

        let keymap =
            KeyMap::try_from(keymap_node).expect("default config should always be a valid keymap");

        Self {
            app_config: AppConfig {
                data_dir: get_data_dir(),
                config_dir: get_config_dir(),
            },
            keymap,
        }
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
