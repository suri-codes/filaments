//! Filaments
//! My (suri.codes) personal-knowledge-system, with deeply integrated task tracking and long term goal planning capabilities.
//!

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{cli::Cli, gui::FilViz, tui::TuiApp};
use clap::Parser;

mod cli;
mod gui;
mod tui;

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
        return rt.block_on(async { command.process() });
    }

    let shutdown_signal = Arc::new(AtomicBool::new(false));

    // then we spawn the tui on its own thread
    let tui_handle = std::thread::spawn({
        let tui_rt = rt.clone();
        let shutdown = shutdown_signal.clone();
        move || -> color_eyre::Result<()> {
            // block the tui on the same runtime as above
            tui_rt.block_on(async {
                let mut tui = TuiApp::new(args.tick_rate, args.frame_rate)?;
                tui.run().await?;
                shutdown.store(true, Ordering::Relaxed);
                Ok(())
            })
        }
    });

    // if they asked for the visualizer, we give them the visualizer
    if args.visualizer {
        // enter the guard so egui_async works properly
        let _rt_guard = rt.enter();
        FilViz::run(shutdown_signal)?;
    }

    // join on the tui
    tui_handle.join().unwrap()?;
    Ok(())
}
