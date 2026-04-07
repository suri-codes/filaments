use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::types::Zettel;

/// A `Widget` that represents a `Zettel`
#[derive(Debug, Clone)]
pub struct ZettelView<'text> {
    // title: Text<'text>,
    title: Paragraph<'text>,
    tags: Paragraph<'text>,
    created_at: Paragraph<'text>,
    zettel_id: Paragraph<'text>,
    layouts: Layouts,
}

impl From<&Zettel> for ZettelView<'_> {
    fn from(value: &Zettel) -> Self {
        Self {
            title: Paragraph::new(value.title.clone()).block(
                Block::default()
                    .title("Title")
                    .borders(Borders::all())
                    .border_style(Style::new())
                    .border_type(BorderType::Plain),
            ),
            tags: Paragraph::new(
                value
                    .tags
                    .iter()
                    .map(|t| {
                        Span::from(format!("{} ", t.name)).style(Style::new().fg(t.color.into()))
                    })
                    .collect::<Line>(),
            )
            .block(
                Block::default()
                    .title("Tags")
                    .borders(Borders::all())
                    .border_style(Style::new())
                    .border_type(BorderType::Plain),
            ),
            created_at: Paragraph::new(value.created_at()).block(
                Block::default()
                    .title("Created At")
                    .borders(Borders::all())
                    .border_style(Style::new())
                    .border_type(BorderType::Plain),
            ),
            zettel_id: Paragraph::new(value.id.to_string()).block(
                Block::default()
                    .title("Id")
                    .borders(Borders::all())
                    .border_style(Style::new())
                    .border_type(BorderType::Plain),
            ),

            layouts: Layouts::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct Layouts {
    top_bottom: Layout,
    title_created: Layout,
    tags_id: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            top_bottom: Layout::vertical(vec![Constraint::Min(3), Constraint::Min(3)]),

            title_created: Layout::horizontal(vec![Constraint::Fill(80), Constraint::Min(24)]),

            tags_id: Layout::horizontal(vec![Constraint::Fill(95), Constraint::Min(10)]),
        }
    }
}

impl Widget for ZettelView<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (title_rect, created_rect, tags_rect, modified_rect) = {
            let rects = self.layouts.top_bottom.split(area);

            let (left, right) = (rects[0], rects[1]);

            let t_rects = self.layouts.title_created.split(left);

            let b_rects = self.layouts.tags_id.split(right);

            (t_rects[0], t_rects[1], b_rects[0], b_rects[1])
        };

        self.title.render(title_rect, buf);
        self.tags.render(tags_rect, buf);
        self.created_at.render(created_rect, buf);
        self.zettel_id.render(modified_rect, buf);
    }
}
