use ratatui::{
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

#[derive(Debug, Clone)]
pub struct Preview<'text> {
    content: Paragraph<'text>,
}

impl From<String> for Preview<'_> {
    fn from(value: String) -> Self {
        Self {
            content: Paragraph::new(Text::from(value)).block(
                Block::new()
                    .borders(Borders::TOP | Borders::LEFT)
                    .border_type(BorderType::Rounded)
                    .title("Preview"),
            ),
        }
    }
}

impl Widget for Preview<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.content.render(area, buf);
    }
}
