//! Filaments
//! My (suri.codes) personal-knowledge-system, with deeply integrated task tracking and long term goal planning capabilities.
//!

use crate::{app::App, cli::Cli};
use clap::Parser;
use db::Db;

mod app;
mod cli;
mod components;
mod config;
mod errors;
mod keymap;
mod logging;
mod signal;
mod tui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    errors::init()?;
    logging::init()?;

    let args = Cli::parse();

    let _db = Db::connect("/tmp/filaments/test_db.sqlite").await?;

    // if there is any subcommand, we want to execute that, otherwise we
    // just run the app

    if let Some(command) = args.command {
        match command {
            cli::Commands::Test => {}
        }
    } else {
        let mut app = App::new(args.tick_rate, args.frame_rate);

        app.run().await?;
    }
    Ok(())
}
