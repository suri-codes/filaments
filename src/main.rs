//! Filaments
//! My (suri.codes) personal-knowledge-system, with deeply integrated task tracking and long term goal planning capabilities.
//!

mod app;
mod components;
mod config;
mod errors;
mod logging;
mod signal;
mod tui;

fn main() -> color_eyre::Result<()> {
    errors::init()?;
    logging::init()?;

    Ok(())
}
