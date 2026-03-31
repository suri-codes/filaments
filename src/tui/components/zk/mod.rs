use async_trait::async_trait;
use color_eyre::{eyre::Result, owo_colors::colors::Red};
use ratatui::{prelude::*, widgets::Block};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    tui::{Signal, components::Component},
    types::KastenHandle,
};

mod zettel;

pub struct Zk {
    signal_tx: Option<UnboundedSender<Signal>>,
    kh: KastenHandle,
    layouts: Layouts,
}

struct Layouts {
    left_right: Layout,
    search_zl: Layout,
    z_preview: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            left_right: Layout::horizontal(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]),
            search_zl: Layout::vertical(vec![
                Constraint::Percentage(15),
                Constraint::Percentage(85),
            ]),
            z_preview: Layout::vertical(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ]),
        }
    }
}

impl Zk {
    pub fn new(kh: KastenHandle) -> Self {
        Self {
            signal_tx: None,
            kh,
            layouts: Layouts::default(),
        }
    }
}

#[async_trait]
impl Component for Zk {
    fn register_signal_handler(&mut self, tx: UnboundedSender<Signal>) -> Result<()> {
        self.signal_tx = Some(tx);
        Ok(())
    }

    async fn update(&mut self, _signal: Signal) -> Result<Option<crate::tui::Signal>> {
        // match signal {
        //     Signal::Tick => todo!(),
        //     Signal::Render => todo!(),
        //     Signal::Resize(_, _) => todo!(),
        //     Signal::Suspend => todo!(),
        //     Signal::Resume => todo!(),
        //     Signal::Quit => todo!(),
        //     Signal::ClearScreen => todo!(),
        //     Signal::Error(_) => todo!(),
        //     Signal::Help => todo!(),
        //     Signal::Helix => todo!(),
        // }
        Ok(None)
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        let (search_layout, zettel_list_layout, zettel_layout, preview_layout) = {
            let rects = self.layouts.left_right.split(area);

            let (left, right) = (rects[0], rects[1]);

            let l_rects = self.layouts.search_zl.split(left);

            let r_rects = self.layouts.z_preview.split(right);

            (l_rects[0], l_rects[1], r_rects[0], r_rects[1])
        };

        frame.render_widget(Block::new().bg(Color::Red), search_layout);
        frame.render_widget(Block::new().bg(Color::Blue), zettel_list_layout);
        frame.render_widget(Block::new().bg(Color::Green), zettel_layout);
        frame.render_widget(Block::new().bg(Color::Yellow), preview_layout);

        Ok(())
    }
}
