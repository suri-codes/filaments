use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{debug, info};

use crate::{
    components::Component,
    config::Config,
    signal::Signal,
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    #[allow(dead_code)]
    region: Region,
    last_tick_key_events: Vec<KeyEvent>,
    signal_tx: UnboundedSender<Signal>,
    signal_rx: UnboundedReceiver<Signal>,
}

/// The different regions of the application that the user can
/// be interacting with. Think of these kind of like the highest class of
/// components.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Region {
    #[default]
    Home,
}

impl App {
    /// Construct a new `App` instance.
    pub fn new(tick_rate: f64, frame_rate: f64) -> Self {
        let (signal_tx, signal_rx) = mpsc::unbounded_channel();

        Self {
            tick_rate,
            frame_rate,
            components: vec![],
            should_quit: false,
            should_suspend: false,
            config: Config::new(),
            region: Region::default(),
            last_tick_key_events: Vec::new(),
            signal_tx,
            signal_rx,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            .with_tick_rate(self.tick_rate)
            .with_frame_rate(self.frame_rate);
        tui.enter()?;

        for component in &mut self.components {
            component.register_signal_handler(self.signal_tx.clone())?;
        }
        for component in &mut self.components {
            component.register_config_handler(self.config.clone())?;
        }

        for component in &mut self.components {
            component.init(tui.size()?)?;
        }

        let signal_tx = self.signal_tx.clone();

        loop {
            self.handle_events(&mut tui).await?;

            self.handle_signals(&mut tui).await?;
            if self.should_suspend {
                tui.suspend()?;

                // We are sending resume here because once its done suspending,
                // it will continue execution here.
                signal_tx.send(Signal::Resume)?;
                signal_tx.send(Signal::ClearScreen)?;
                tui.enter()?;
            } else if self.should_quit {
                tui.stop();
                break;
            }
        }

        tui.exit()?;

        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };

        let signal_tx = self.signal_tx.clone();

        match event {
            Event::Quit => signal_tx.send(Signal::Quit)?,
            Event::Tick => signal_tx.send(Signal::Tick)?,
            Event::Render => signal_tx.send(Signal::Render)?,
            Event::Resize(x, y) => signal_tx.send(Signal::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,

            _ => {}
        }

        for component in &mut self.components {
            if let Some(signal) = component.handle_events(Some(event.clone()))? {
                signal_tx.send(signal)?;
            }
        }

        Ok(())
    }

    // We are okay with this because we know that this is the function signature,
    // we just haven't implemented the keyboard parsing logic just yet, revisit
    // this later.
    //
    // DO NOT LET THIS MERGE INTO MAIN WITH THIS CLIPPY IGNORES
    #[allow(clippy::needless_pass_by_ref_mut, clippy::unnecessary_wraps)]
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let _signal_tx = self.signal_tx.clone();

        info!("key received: {key:#?}");

        Ok(())
    }

    async fn handle_signals(&mut self, tui: &mut Tui) -> Result<()> {
        while let Some(signal) = self.signal_rx.recv().await {
            if signal != Signal::Tick && signal != Signal::Render {
                debug!("App: handling signal: {signal:?}");
            }

            match signal {
                Signal::Tick => {
                    self.last_tick_key_events.drain(..);
                }

                Signal::Quit => self.should_quit = true,

                Signal::Suspend => self.should_suspend = true,

                Signal::Resume => self.should_suspend = false,

                Signal::ClearScreen => tui.terminal.clear()?,
                Signal::Resize(x, y) => self.handle_resize(tui, x, y)?,
                Signal::Render => self.render(tui)?,
                _ => {}
            }

            for component in &mut self.components {
                if let Some(signal) = component.update(signal.clone())? {
                    self.signal_tx.send(signal)?;
                }
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;

        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            for component in &mut self.components {
                if let Err(err) = component.draw(frame, frame.area()) {
                    let _ = self
                        .signal_tx
                        .send(Signal::Error(format!("Failed to draw: {err:?}")));
                }
            }
        })?;

        Ok(())
    }
}
