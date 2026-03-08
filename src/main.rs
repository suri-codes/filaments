//! Filaments
//! My (suri.codes) personal-knowledge-system, with deeply integrated task tracking and long term goal planning capabilities.
//!

use clap::Parser;

use crate::{app::App, cli::Cli};

mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod signal;
mod tui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    errors::init()?;
    logging::init()?;

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate);

    app.run().await?;

    Ok(())
}
