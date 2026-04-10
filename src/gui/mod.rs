use eframe::egui;
use egui_graphs::{SettingsInteraction, SettingsNavigation};

use crate::types::FilamentsHandle;

/// The `Filaments Visualizer`, which is an instance of `eframe`, which uses `egui`
pub struct FilViz {
    fh: FilamentsHandle,
}

impl FilViz {
    /// Create a new instance of the `FiLViz`
    const fn new(_cc: &eframe::CreationContext<'_>, fh: FilamentsHandle) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self { fh }
    }

    /// Create and run the `FilViz`.
    pub fn run(fh: FilamentsHandle) -> color_eyre::Result<()> {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "Filaments Visualizer",
            native_options,
            Box::new(|cc| Ok(Box::new(Self::new(cc, fh)))),
        )?;

        Ok(())
    }
}

type L = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;

impl eframe::App for FilViz {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let g = &mut self.fh.lock().expect("Lock must not be poisoned").graph;

            let mut view = egui_graphs::GraphView::<_, _, _, _, _, _, S, L>::new(g)
                .with_interactions(
                    &SettingsInteraction::new()
                        .with_hover_enabled(false)
                        .with_dragging_enabled(false)
                        .with_node_selection_enabled(false)
                        .with_node_selection_multi_enabled(false)
                        .with_edge_selection_enabled(false)
                        .with_edge_selection_multi_enabled(false),
                )
                .with_navigations(
                    &SettingsNavigation::new().with_fit_to_screen_padding(0.2), // .with_zoom_and_pan_enabled(true)
                                                                                // .with_fit_to_screen_enabled(false),
                );

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
