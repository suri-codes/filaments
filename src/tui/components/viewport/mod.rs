use async_trait::async_trait;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Size},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::debug;

use crate::{
    tui::{
        Signal,
        app::Page,
        components::{Component, Todo, Zk},
    },
    types::KastenHandle,
};

pub struct Viewport<'text> {
    signal_tx: Option<UnboundedSender<Signal>>,
    #[expect(dead_code)]
    kh: KastenHandle,
    _layouts: Layouts,
    switcher: Switcher<'text>,
    active_page: Page,
    zk: Zk<'text>,
    todo: Todo<'text>,
}

mod switcher;
use switcher::Switcher;

impl Viewport<'_> {
    pub fn new(kh: KastenHandle) -> Self {
        let mut switcher = Switcher::default();
        switcher.select_region(Page::default());

        Self {
            signal_tx: None,
            _layouts: Layouts::default(),
            switcher,
            zk: Zk::new(kh.clone()),
            todo: Todo::new(kh.clone()),
            active_page: Page::default(),
            kh,
        }
    }
}

struct Layouts {
    _main_switcher: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            _main_switcher: Layout::vertical(vec![Constraint::Fill(90), Constraint::Min(1)]),
        }
    }
}

#[async_trait]
impl Component for Viewport<'_> {
    async fn init(&mut self, area: Size) -> color_eyre::Result<()> {
        self.zk.init(area).await?;
        self.todo.init(area).await?;

        self.signal_tx.as_mut().unwrap().send(Signal::SwitchTo {
            page: Page::default(),
        })?;

        Ok(())
    }

    fn register_signal_handler(&mut self, tx: UnboundedSender<Signal>) -> Result<()> {
        self.signal_tx = Some(tx.clone());
        self.zk.register_signal_handler(tx.clone())?;
        self.todo.register_signal_handler(tx)?;
        Ok(())
    }

    async fn update(&mut self, signal: Signal) -> color_eyre::Result<Option<Signal>> {
        // switch active region
        if let Signal::SwitchTo { page } = signal {
            self.active_page = page;
            self.switcher.select_region(page);
            debug!("active page switched to : {page}");
        }

        if let Some(signal) = self.zk.update(signal.clone()).await? {
            self.signal_tx.as_mut().unwrap().send(signal)?;
        }
        self.todo.update(signal).await
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Signal>> {
        match self.active_page {
            Page::Zk => self.zk.handle_key_event(key).await,
            Page::Todo(_) => self.todo.handle_key_event(key).await,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        // figure out how we are to do this after
        // let (main_layout, _switcher_layout) = {
        //     let rects = self.layouts.main_switcher.split(area);
        //     (rects[0], rects[1])
        // };
        //

        match self.active_page {
            Page::Zk => self.zk.draw(frame, area),
            Page::Todo(_) => self.todo.draw(frame, area),
        }?;

        Ok(())
    }
}
