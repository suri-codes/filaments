//! Filaments
//! My (suri.codes) personal-knowledge-system, with deeply integrated task tracking and long term goal planning capabilities.
//!

use std::{process, sync::Arc};

use crate::{cli::Cli, gui::FilViz, tui::TuiApp};
use clap::Parser;

mod cli;
mod gui;
mod tui;
mod types;

mod config;
mod errors;
mod logging;

fn main() -> color_eyre::Result<()> {
    errors::init()?;
    logging::init()?;

    let args = Cli::parse();

    // create a new tokio runtime, shared behind arc
    let rt = Arc::new(tokio::runtime::Runtime::new()?);

    // if there are any commands, run those and exit
    if let Some(command) = args.command {
        return rt.block_on(async { command.process().await });
    }

    // then we spawn the tui on its own thread
    let tui_handle = std::thread::spawn({
        // arc stuff
        let tui_rt = rt.clone();

        // closure to run the tui
        move || -> color_eyre::Result<()> {
            // block the tui on the same runtime as above
            tui_rt.block_on(async {
                let mut tui = TuiApp::new(args.tick_rate, args.frame_rate)?;
                tui.run().await?;
                // just close everything as soon as the tui is done running
                process::exit(0);
            })
        }
    });

    // if they asked for the visualizer, we give them the visualizer
    if args.visualizer {
        // enter the guard so egui_async works properly
        let _rt_guard = rt.enter();
        FilViz::run()?;
    }

    // join on the tui
    tui_handle.join().unwrap()?;
    Ok(())
}
