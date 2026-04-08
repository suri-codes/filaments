use eframe::egui;

use crate::types::{Filaments, Index};

/// The `Filaments Visualizer`, which is an instance of `eframe`, which uses `egui`
pub struct FilViz {
    filaments: Filaments,
}

impl FilViz {
    /// Create a new instance of the `FiLViz`
    fn new(_cc: &eframe::CreationContext<'_>, idx: &Index) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            filaments: Filaments::from(idx),
        }
    }

    /// Create and run the `FilViz`.
    pub fn run(idx: &Index) -> color_eyre::Result<()> {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "Filaments Visualizer",
            native_options,
            Box::new(|cc| Ok(Box::new(Self::new(cc, idx)))),
        )?;

        Ok(())
    }
}

type L = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;

impl eframe::App for FilViz {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let g = &mut self.filaments.graph;

            let mut view = egui_graphs::GraphView::<_, _, _, _, _, _, S, L>::new(g);

            ui.add(&mut view);

            // credits!
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
