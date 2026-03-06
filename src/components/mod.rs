use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Rect, Size},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{config::Config, signal::Signal, tui::Event};

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementers of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
#[expect(dead_code)]
pub trait Component {
    /// Register a signal handler that can send signals for processing if necessary.
    ///
    /// # Arguments
    ///
    /// * `tx` - An unbounded sender that can send signals.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<()>`] - An Ok result or an error.
    fn register_signal_handler(&mut self, tx: UnboundedSender<Signal>) -> color_eyre::Result<()> {
        let _ = tx; // to appease clippy
        Ok(())
    }
    /// Register a configuration handler that provides configuration settings if necessary.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<()>`] - An Ok result or an error.
    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        let _ = config; // to appease clippy
        Ok(())
    }
    /// Initialize the component with a specified area if necessary.
    ///
    /// # Arguments
    ///
    /// * `area` - Rectangular area to initialize the component within.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<()>`] - An Ok result or an error.
    fn init(&mut self, area: Size) -> color_eyre::Result<()> {
        let _ = area; // to appease clippy
        Ok(())
    }
    /// Handle incoming events and produce signals if necessary.
    ///
    /// # Arguments
    ///
    /// * `event` - An optional event to be processed.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<Option<signal>>`] - A signal to be processed or none.
    fn handle_events(&mut self, event: Option<Event>) -> color_eyre::Result<Option<Signal>> {
        let signal = match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
            _ => None,
        };
        Ok(signal)
    }
    /// Handle key events and produce signals if necessary.
    ///
    /// # Arguments
    ///
    /// * `key` - A key event to be processed.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<Option<signal>>`] - A signal to be processed or none.
    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Signal>> {
        let _ = key; // to appease clippy
        Ok(None)
    }
    /// Handle mouse events and produce signals if necessary.
    ///
    /// # Arguments
    ///
    /// * `mouse` - A mouse event to be processed.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<Option<signal>>`] - A signal to be processed or none.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> color_eyre::Result<Option<Signal>> {
        let _ = mouse; // to appease clippy
        Ok(None)
    }
    /// Update the state of the component based on a received signal. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `signal` - A signal that may modify the state of the component.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<Option<signal>>`] - A signal to be processed or none.
    fn update(&mut self, signal: Signal) -> color_eyre::Result<Option<Signal>>;

    /// Render the component on the screen. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the component should be drawn.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<()>`] - An Ok result or an error.
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()>;
}
