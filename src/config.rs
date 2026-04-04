use color_eyre::eyre::Context;
use directories::ProjectDirs;
use kdl::KdlDocument;
use serde::Deserialize;
use std::{
    env::{self, home_dir},
    fmt::Debug,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crate::tui::KeyMap;

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
    /// The directory where the single instance of the filaments exists.
    pub workspace: PathBuf,
    #[serde(default)]
    pub data: PathBuf,
    #[serde(default)]
    pub config: PathBuf,
}

/// Configuration for the App
#[derive(Debug, Clone)]
pub struct Config {
    pub app_config: AppConfig,
    pub keymap: KeyMap,
    // pub styles: Styles,
}

impl Config {
    /// generates a new config with the provided `filaments_dir`
    pub fn generate(filaments_dir: &Path) -> KdlDocument {

        
        let mut default_config: KdlDocument = DEFAULT_CONFIG
            .parse()
            .expect("Default config should always be a valid KDL document.");

        if let Some(node) = default_config
            .nodes_mut()
            .iter_mut()
            .find(|n| n.name().value() == "filaments_dir")
            && let Some(entry) = node.entries_mut().get_mut(0)
        {
            *entry.value_mut() = kdl::KdlValue::String(filaments_dir.to_string_lossy().to_string());
            entry.clear_format();
        }

        default_config
    }

    /// Parse the config from `~/.config/filametns`
    ///
    /// # Errors
    ///
    /// Will error if the config doesn't exist or if there
    /// is a problem parsing it.
    pub fn parse() -> color_eyre::Result<Self> {
        let config: KdlDocument = {
            let file_path = get_config_dir().join("config.kdl");

            let mut file = File::open(file_path).context("Failed to find file!")?;

            let mut str = String::new();

            file.read_to_string(&mut str)
                .context("Failed to read file!")?;

            str.parse().context("Expected to be valid kdl")?
        };

        let keymap = KeyMap::try_from(
            config
                .get("keymap")
                .expect("Keymap must exist in the config"),
        )
        .context("Keymap is not valid!")?;

        let filaments_dir_str = config
            .get("filaments_dir")
            .expect("config should always have this specified")
            .get(0)
            .and_then(|arg| arg.as_string())
            .expect("filaments_dir must be a string");

        let filaments_dir = PathBuf::from(filaments_dir_str)
            .canonicalize()
            .context("Filaments directory does not exist!")?;

        Ok(Self {
            app_config: AppConfig {
                workspace: filaments_dir,
                data: get_data_dir(),
                config: get_config_dir(),
            },
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
