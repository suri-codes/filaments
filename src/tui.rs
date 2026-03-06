use std::{
    io::{Stdout, stdout},
    ops::{Deref, DerefMut},
    time::Duration,
};

use color_eyre::eyre::Result;
use crossterm::{
    cursor,
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        EventStream, KeyEvent, KeyEventKind, MouseEvent,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode},
};
use futures::{FutureExt as _, StreamExt as _};
use ratatui::{Terminal, prelude::CrosstermBackend};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
    time::interval,
};
use tokio_util::sync::CancellationToken;
use tracing::error;

/// Events processed by the whole application.
#[expect(dead_code)]
pub enum Event {
    /// Application initialized
    Init,

    /// Application quit
    Quit,

    /// Application error
    Error,

    /// Application closed
    Closed,

    /// Tick = input refresh rate
    Tick,

    /// Render application
    Render,

    /// User enters application
    FocusGained,

    /// User leaves application
    FocusLost,

    /// Paste buffer
    Paste(String),

    /// any key event
    Key(KeyEvent),

    /// any mouse event
    Mouse(MouseEvent),

    /// Application resize
    Resize(u16, u16),
}

/// A TUI which supports the general things you would want out of a TUI abstraction.
pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub task: JoinHandle<()>,
    pub cancellation_token: CancellationToken,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub frame_rate: f64,
    pub tick_rate: f64,
    pub mouse_enabled: bool,
    pub paste_enabled: bool,
}

#[expect(dead_code)]
impl Tui {
    /// Creates a new TUI.
    pub fn new() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(stdout()))?,
            task: tokio::spawn(async {}),
            cancellation_token: CancellationToken::new(),
            event_rx,
            event_tx,
            frame_rate: 60.0,
            tick_rate: 4.0,
            mouse_enabled: false,
            paste_enabled: false,
        })
    }

    /// Set the tick rate, which is how often the TUI should
    /// source events per second.
    pub const fn with_tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    /// Set the frame rate.
    pub const fn with_frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    /// Enable mouse interactions.
    pub const fn set_mouse_enable(mut self, mouse_enabled: bool) -> Self {
        self.mouse_enabled = mouse_enabled;
        self
    }

    /// Enable pasting into the TUI.
    pub const fn set_paste_enabled(mut self, paste_enabled: bool) -> Self {
        self.paste_enabled = paste_enabled;
        self
    }

    /// Begin the TUI event loop
    pub fn start(&mut self) {
        self.cancel();

        self.cancellation_token = CancellationToken::new();
        let event_loop = Self::event_loop(
            self.event_tx.clone(),
            self.cancellation_token.clone(),
            self.tick_rate,
            self.frame_rate,
        );
        self.task = tokio::spawn(async {
            event_loop.await;
        });
    }

    /// The event-loop for the TUI which sources events from crossterm.
    async fn event_loop(
        event_tx: UnboundedSender<Event>,
        cancellation_token: CancellationToken,
        tick_rate: f64,
        frame_rate: f64,
    ) {
        use crossterm::event::Event as CrosstermEvent;

        let mut event_stream = EventStream::new();
        let mut tick_interval = interval(Duration::from_secs_f64(1.0 / tick_rate));
        let mut render_interval = interval(Duration::from_secs_f64(1.0 / frame_rate));

        event_tx
            .send(Event::Init)
            .expect("Tui::event_loop: Failed to send init event.");
        loop {
            let event = tokio::select! {
                () = cancellation_token.cancelled() => {
                    break;
                }
                _ = tick_interval.tick() => Event::Tick,
                _ = render_interval.tick() => Event::Render,
                crossterm_event = event_stream.next().fuse() => match crossterm_event {
                    Some(Ok(event)) => match event {
                        // we only care about press down events,
                        // not doing anything related to up / down keypresses
                        CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => Event::Key(key),
                        CrosstermEvent::Key(_) => continue,

                        CrosstermEvent::Mouse(mouse) => Event::Mouse(mouse),
                        CrosstermEvent::Resize(x, y) => Event::Resize(x, y),
                        CrosstermEvent::FocusLost => {Event::FocusLost },
                        CrosstermEvent::FocusGained => {Event::FocusGained },
                        CrosstermEvent::Paste(s)=> {Event::Paste(s)},

                    }
                    Some(Err(_)) => Event::Error,
                    None => break,
                }
            };
            if event_tx.send(event).is_err() {
                // no more receiver
                break;
            }
        }

        cancellation_token.cancel();
    }

    /// Stops the TUI by canceling the event-loop.
    pub fn stop(&self) {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                error!("Failed to abort task in 100 milliseconds for some reason");
                break;
            }
        }
    }

    // Enters into the TUI by enabling alternate screen and starting event-loop.
    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        if self.mouse_enabled {
            crossterm::execute!(stdout(), EnableMouseCapture)?;
        }
        if self.paste_enabled {
            crossterm::execute!(stdout(), EnableBracketedPaste)?;
        }
        self.start();
        Ok(())
    }

    /// Exits the tui, by leaving alternate screen.
    pub fn exit(&mut self) -> color_eyre::Result<()> {
        self.stop();
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            if self.paste_enabled {
                crossterm::execute!(stdout(), DisableBracketedPaste)?;
            }
            if self.mouse_enabled {
                crossterm::execute!(stdout(), DisableMouseCapture)?;
            }
            crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    /// Cancel the internal event-loop.
    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    /// Suspend the TUI.
    pub fn suspend(&mut self) -> color_eyre::Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    /// Resume the TUI.
    pub fn resume(&mut self) -> color_eyre::Result<()> {
        self.enter()?;
        Ok(())
    }

    /// Get the next event.
    pub async fn next_event(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
