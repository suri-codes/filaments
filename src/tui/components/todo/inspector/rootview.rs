use ratatui::widgets::{Paragraph, Widget};

#[derive(Debug, Clone)]
pub struct RootView<'text> {
    help: Paragraph<'text>,
}

impl Default for RootView<'_> {
    fn default() -> Self {
        Self {
            help: Paragraph::new("Try making a selection!"),
        }
    }
}

impl Widget for RootView<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.help.render(area, buf);
    }
}
