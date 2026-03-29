use async_trait::async_trait;
use color_eyre::eyre::Result;
use ratatui::widgets::Paragraph;
use tokio::sync::mpsc::UnboundedSender;

use crate::tui::{Signal, components::Component};

mod zettel;

pub struct Zk {
    signal_tx: Option<UnboundedSender<Signal>>,
}

impl Zk {
    pub const fn new() -> Self {
        Self { signal_tx: None }
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
        let hello = Paragraph::new("Hello Surendra");
        frame.render_widget(hello, area);
        Ok(())
    }
}
