use async_trait::async_trait;
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

pub struct Todo {
    signal_tx: Option<UnboundedSender<Signal>>,
    kh: KastenHandle,
    layouts: Layouts,
}

struct Layouts {
    main: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            main: Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)]),
        }
    }
}

#[async_trait]
impl Component for Todo {
    async fn update(&mut self, signal: Signal) -> color_eyre::Result<Option<Signal>> {
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        frame.render_widget(Block::new().fg(Color::Red), area);
        Ok(())
    }
}
