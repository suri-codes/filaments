use std::{
    process,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use eframe::egui;

/// The `Filaments Visualizer`, which is an instance of `eframe`, which uses `egui`
#[derive(Default)]
pub struct FilViz {
    shutdown_signal: Arc<AtomicBool>,
    /// example for now
    text: String,
}

impl FilViz {
    /// Create a new instance of the `FiLViz`
    const fn new(_cc: &eframe::CreationContext<'_>, shutdown_signal: Arc<AtomicBool>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            shutdown_signal,
            text: String::new(),
        }
    }

    /// Create and run the `FilViz`.
    pub fn run(shutdown_signal: Arc<AtomicBool>) -> color_eyre::Result<()> {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "Filaments Visualizer",
            native_options,
            Box::new(|cc| Ok(Box::new(Self::new(cc, shutdown_signal)))),
        )?;

        Ok(())
    }
}

impl eframe::App for FilViz {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.shutdown_signal.load(Ordering::Relaxed) {
            process::exit(0)
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Hello World!");
            ui.text_edit_singleline(&mut self.text);
        });
    }
}
