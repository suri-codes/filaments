use std::{process::Command, thread::spawn};

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::debug;

use crate::{
    config::Config,
    tui::{Event, Tui, components::Zk},
    types::KastenHandle,
};

use super::{components::Component, signal::Signal};

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
    kh: KastenHandle,
    signal_tx: UnboundedSender<Signal>,
    signal_rx: UnboundedReceiver<Signal>,
}

/// The different regions of the application that the user can
/// be interacting with. Think of these kind of like the highest class of
/// components.
#[derive(
    Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display,
)]
pub enum Region {
    #[default]
    Home,
    Zk,
}

impl App {
    /// Construct a new `App` instance.
    pub fn new(tick_rate: f64, frame_rate: f64, kh: KastenHandle) -> Result<Self> {
        let (signal_tx, signal_rx) = mpsc::unbounded_channel();

        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(Zk::new(kh.clone()))],
            should_quit: false,
            should_suspend: false,
            config: Config::parse()?,
            region: Region::default(),
            last_tick_key_events: Vec::new(),
            kh,
            signal_tx,
            signal_rx,
        })
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

        debug!("received event: {event:?}");

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

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        debug!("key received: {key:#?}");

        let signal_tx = self.signal_tx.clone();

        let Some(region_keymap) = self.config.keymap.get(&self.region) else {
            return Ok(());
        };

        if let Some(signal) = region_keymap.get(&vec![key]) {
            signal_tx.send(signal.clone())?;
        } else {
            self.last_tick_key_events.push(key);
            if let Some(signal) = region_keymap.get(&self.last_tick_key_events) {
                debug!("Got signal: {signal:?}");
                signal_tx.send(signal.clone())?;
            }
        }

        Ok(())
    }

    async fn handle_signals(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(signal) = self.signal_rx.try_recv() {
            if signal != Signal::Tick && signal != Signal::Render {
                debug!("handling signal: {signal:?}");
            }

            match signal {
                Signal::Tick => {
                    self.last_tick_key_events.drain(..);
                }

                Signal::Quit => self.should_quit = true,

                Signal::Helix => {
                    tui.exit()?;

                    let hx = spawn(move || -> Result<()> {
                        Command::new("hx")
                            .stdin(std::process::Stdio::inherit())
                            .stdout(std::process::Stdio::inherit())
                            .stderr(std::process::Stdio::inherit())
                            .status()?;

                        Ok(())
                    });

                    hx.join().unwrap().unwrap();

                    tui.terminal.clear()?;
                    tui.enter()?;
                }

                Signal::Suspend => self.should_suspend = true,

                Signal::Resume => self.should_suspend = false,

                Signal::ClearScreen => tui.terminal.clear()?,
                Signal::Resize(x, y) => self.handle_resize(tui, x, y)?,
                Signal::Render => self.render(tui)?,
                _ => {}
            }

            for component in &mut self.components {
                if let Some(signal) = component.update(signal.clone()).await? {
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
