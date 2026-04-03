use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Widget},
};
use ratatui_textarea::TextArea;

#[derive(Clone)]
pub struct Search<'text> {
    pub title: TextArea<'text>,
    pub tag: TextArea<'text>,
    layouts: Layouts,
}

impl Default for Search<'_> {
    fn default() -> Self {
        let mut title = TextArea::default();

        title.set_style(Style::default());
        title.set_block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all())
                .title("Search Titles"),
        );

        let mut tag = TextArea::default();
        tag.set_style(Style::default());
        tag.set_block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all())
                .title("Search Tags"),
        );

        Self {
            title,
            tag,
            layouts: Layouts::default(),
        }
    }
}

#[derive(Clone)]
struct Layouts {
    title_tag: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            title_tag: Layout::vertical(vec![Constraint::Min(3), Constraint::Min(3)]),
        }
    }
}

impl Widget for Search<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (title_search_rect, tag_search_rect) = {
            let rects = self.layouts.title_tag.split(area);

            (rects[0], rects[1])
        };

        self.title.render(title_search_rect, buf);
        self.tag.render(tag_search_rect, buf);
    }
}
