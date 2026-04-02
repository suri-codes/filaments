use ratatui::{style::Style, text::Text, widgets::Widget};

#[derive(Debug, Clone)]
pub struct Preview<'text> {
    content: Text<'text>,
}

impl From<String> for Preview<'_> {
    fn from(value: String) -> Self {
        Self {
            content: Text::from(value),
        }
    }
}

impl Widget for Preview<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.content.style(Style::new()).render(area, buf);
    }
}
