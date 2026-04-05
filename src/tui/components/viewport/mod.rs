use async_trait::async_trait;
use color_eyre::eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::Block,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    tui::{Signal, components::Component},
    types::KastenHandle,
};

pub struct Viewport {
    signal_tx: Option<UnboundedSender<Signal>>,
    kh: KastenHandle,
    layouts: Layouts,
}

impl Viewport {
    pub async fn new(kh: KastenHandle) -> Result<Self> {
        let kt = kh.read().await;
        drop(kt);
        Ok(Self {
            signal_tx: None,
            kh,
            layouts: Layouts::default(),
        })
    }
}

struct Layouts {
    main_switcher: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            main_switcher: Layout::vertical(vec![Constraint::Fill(90), Constraint::Min(1)]),
        }
    }
}

#[async_trait]
impl Component for Viewport {
    async fn update(&mut self, signal: Signal) -> color_eyre::Result<Option<Signal>> {
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let (main_layout, switcher_layout) = {
            let rects = self.layouts.main_switcher.split(area);
            (rects[0], rects[1])
        };

        // frame.render_widget(Block::new().fg(Color::Red), main_layout);
        // frame.render_widget(Block::new().fg(Color::Green), switcher_layout);
        //
        frame.render_widget(Block::new().bg(Color::Green), main_layout);
        frame.render_widget(Block::new().bg(Color::Yellow), switcher_layout);
        Ok(())
    }
}
