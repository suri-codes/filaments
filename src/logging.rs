use std::{
    fs::{File, create_dir_all},
    sync::LazyLock,
};

use color_eyre::eyre::Result;
use tracing::{Level, info};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config;

/// The user-set log level if it exists.
pub static LOG_LEVEL_ENV: LazyLock<String> =
    LazyLock::new(|| format!("{}_LOG_LEVEL", config::PROJECT_NAME.clone()));

/// The logfile name set by our package name.
pub static LOG_FILE: LazyLock<String> = LazyLock::new(|| format!("{}.log", env!("CARGO_PKG_NAME")));

/// Initializes the logger, which writes logs to a `log_file` in the data dir.
///
/// NOTE: log level is configurable via the `RUST_LOG` env var or the
/// `FILAMENTS_LOG_LEVEL` env var
pub fn init() -> Result<()> {
    let directory = config::get_data_dir();

    info!("{directory:#?}");

    create_dir_all(&directory)?;

    let log_path = directory.join(LOG_FILE.clone());
    let log_file = File::create(log_path)?;

    let env_filter = EnvFilter::builder().with_default_directive(Level::INFO.into());

    // If `RUST_LOG` is set, use that as default,, or use value of `LOG_ENV` variable.
    let env_filter = env_filter
        .try_from_env()
        .or_else(|_| env_filter.with_env_var(LOG_LEVEL_ENV.clone()).from_env())?;

    let file_subscriber = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(env_filter);
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .try_init()?;

    Ok(())
}
