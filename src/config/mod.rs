use std::{
    env::{self, home_dir},
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use color_eyre::eyre::{Context, Result};
use directories::ProjectDirs;
use ron::ser::PrettyConfig;

use crate::config::{file::RonConfig, keymap::KeyMap};

mod file;
mod keymap;

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

const DEFAULT_CONFIG: &str = include_str!("../../.config/default_config.ron");

#[derive(Debug, Clone)]
pub struct Config {
    pub fil_dir: PathBuf,
    pub keymap: KeyMap,
}

impl Config {
    /// generates a new config with the provided `filaments_dir`
    pub fn generate(fil_dir: &Path) -> Result<String> {
        let mut default_conf: RonConfig = ron::from_str(DEFAULT_CONFIG)
            .expect("The default config must always be a valid RonConfig");

        default_conf.directory = fil_dir.canonicalize()?;

        Ok(ron::ser::to_string_pretty(
            &default_conf,
            PrettyConfig::default(),
        )?)
    }
    /// Parse the config from `~/.config/filaments`, but will prioritize
    /// `FIL_CONFIG_DIR`.
    ///
    /// # Errors
    ///
    /// Will error if the config doesn't exist or if there
    /// is a problem parsing it.
    pub fn parse() -> Result<Self> {
        let ron: RonConfig = {
            let file_path = get_config_dir().join("config.ron");
            ron::from_str(&read_to_string(&file_path).with_context(|| {
                format!("Failed to read config from path: {}", file_path.display())
            })?)?
        };

        let keymap =
            KeyMap::try_from(&ron).with_context(|| "Unable to parse keymap from config!")?;

        Ok(Self {
            fil_dir: ron
                .directory
                .canonicalize()
                .with_context(|| "Failed to canonicalize the directory provided in the config!")?,
            keymap,
        })
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
        home_dir().map_or_else(
            || PathBuf::from(".").join(".config"),
            |mut path| {
                path.push(".config");
                path.push("filaments");
                path
            },
        )
    })
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "suri-codes", env!("CARGO_PKG_NAME"))
}
