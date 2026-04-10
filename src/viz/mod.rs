use eframe::egui;
use egui_graphs::{SettingsInteraction, SettingsNavigation};
use futures::executor::block_on;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::debug;

use crate::{
    tui::Signal,
    types::{Filaments, Index, KastenHandle},
};

/// The `Filaments Visualizer`, which is an instance of `eframe`, which uses `egui`
pub struct FilViz {
    kh: KastenHandle,
    signal_rx: UnboundedReceiver<Signal>,
    filaments: Filaments,
}

impl FilViz {
    /// Create a new instance of the `FiLViz`
    fn new(
        _cc: &eframe::CreationContext<'_>,
        kh: KastenHandle,
        signal_rx: UnboundedReceiver<Signal>,
        index: &Index,
    ) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            kh,
            signal_rx,
            filaments: index.into(),
        }
    }

    /// Create and run the `FilViz`.
    pub fn run(
        kh: KastenHandle,
        signal_rx: UnboundedReceiver<Signal>,
        index: &Index,
    ) -> color_eyre::Result<()> {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "Filaments Visualizer",
            native_options,
            Box::new(|cc| Ok(Box::new(Self::new(cc, kh, signal_rx, index)))),
        )?;

        Ok(())
    }
}

type L = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;

impl eframe::App for FilViz {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            // let g = &mut self.fh.lock().expect("Lock must not be poisoned").graph;
            let g = &mut self.filaments.graph;

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
                                                                                // .with_fit_to_screen_enabled(false)
                                                                                // ,
                );

            ui.add(&mut view);

            // credits!
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }

    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(signal) = self.signal_rx.try_recv() {
            debug!("received signal in filaments: {signal}");

            #[allow(clippy::single_match)]
            match signal {
                Signal::CreatedZettel { zid } => {
                    block_on(async {
                        let index = &self.kh.read().await.index;
                        self.filaments.insert_zettel(zid, index);
                    });
                }

                Signal::SetLinks { zid, links } => {
                    self.filaments.set_links_for_zid(&zid, links);
                }

                _ => {}
            }
        }
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
