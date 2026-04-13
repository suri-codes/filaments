//! Filaments
//! My (suri.codes) personal-knowledge-system, with deeply integrated task tracking and long term goal planning capabilities.
//!

use std::{process, sync::Arc};

use crate::{
    cli::Cli,
    config::Config,
    tui::TuiApp,
    types::{Deimos, Kasten, KastenHandle},
    viz::FilViz,
};
use clap::Parser;
use tokio::sync::{RwLock, mpsc};
use tracing::debug;

mod cli;
mod config;
mod errors;
mod logging;
mod lsp;
mod tui;
mod types;
mod viz;

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

    // create the kasten handle
    let kh: KastenHandle = rt.block_on(async {
        let cfg = Config::parse()?;

        debug!("Config: {cfg:#?}");

        Ok::<KastenHandle, color_eyre::Report>(Arc::new(RwLock::new(
            Kasten::instansiate(cfg.fil_dir).await?,
        )))
    })?;

    debug!("Kasten Handle: {kh:#?}");

    let (signal_tx, signal_rx) = mpsc::unbounded_channel();

    // then we spawn the tui on its own thread
    let tui_handle = std::thread::spawn({
        // arc stuff
        let tui_rt = rt.clone();
        let kh = kh.clone();
        let signal_tx = signal_tx.clone();

        // closure to run the tui
        move || -> color_eyre::Result<()> {
            // block the tui on the same runtime as above
            tui_rt.block_on(async {
                let mut tui = TuiApp::new(args.tick_rate, args.frame_rate, kh, signal_tx)?;
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

        // spawn deimos
        {
            let kh = kh.clone();
            rt.spawn(async {
                let deimos = Deimos::new(kh, signal_tx);
                deimos.watch().await
            });
        }

        let index = rt.block_on(async { kh.read().await.index.clone() });

        FilViz::run(kh, signal_rx, &index)?;
    }

    // join on the tui
    tui_handle.join().unwrap()?;
    Ok(())
}
