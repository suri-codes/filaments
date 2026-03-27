//! Filaments
//! My (suri.codes) personal-knowledge-system, with deeply integrated task tracking and long term goal planning capabilities.
//!

use crate::{cli::Cli, tui::TuiApp};
use clap::Parser;

mod cli;
mod config;
mod errors;
mod logging;
mod tui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    errors::init()?;
    logging::init()?;

    let args = Cli::parse();

    // if there is any subcommand, we want to execute that, otherwise we
    // just run the app
    if let Some(command) = args.command {
        return command.process();
    }

    // if no command we run the app
    let mut tui = TuiApp::new(args.tick_rate, args.frame_rate)?;
    tui.run().await?;

    Ok(())
}
