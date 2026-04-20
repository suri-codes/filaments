use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, Paragraph, Widget},
};
use ratatui_textarea::TextArea;

use crate::{
    tui::components::preview::Preview,
    types::{Group, Index},
};

#[derive(Debug, Clone)]
pub struct GroupView<'text> {
    pub name: TextArea<'text>,
    pub priority: TextArea<'text>,
    created_at: Paragraph<'text>,
    preview: Preview<'text>,
    layouts: Layouts,
}

#[derive(Debug, Clone)]
struct Layouts {
    left_content: Layout,
    name_priority_created_at: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            left_content: Layout::horizontal(vec![
                Constraint::Percentage(30),
                Constraint::Min(1),
                Constraint::Fill(100),
            ]),
            name_priority_created_at: Layout::vertical(vec![
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Min(3),
            ]),
        }
    }
}

impl From<(&Group, &Index)> for GroupView<'_> {
    fn from(value: (&Group, &Index)) -> Self {
        let group = value.0;
        let idx = value.1;
        let mut name = TextArea::new(vec![group.name.clone()]);
        name.set_block(Block::bordered().title("[N]ame"));
        name.set_cursor_style(Style::reset());
        name.set_cursor_line_style(Style::reset());

        let mut priority = TextArea::new(vec![group.priority.to_string()]);
        priority.set_block(Block::bordered().title("[P]riority"));
        priority.set_cursor_style(Style::reset());
        priority.set_cursor_line_style(Style::reset());

        let preview = idx.get_zod(&group.zettel.id).body.clone().into();

        Self {
            name,
            priority,
            created_at: Paragraph::new(group.created_at())
                .block(Block::bordered().title("Created At")),
            preview,
            layouts: Layouts::default(),
        }
    }
}

impl Widget for GroupView<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (name_rect, priority_rect, created_at, content_rect) = {
            let rects = self.layouts.left_content.split(area);
            let l_rects = self.layouts.name_priority_created_at.split(rects[0]);

            (l_rects[0], l_rects[1], l_rects[2], rects[2])
        };

        self.name.render(name_rect, buf);
        self.priority.render(priority_rect, buf);
        self.created_at.render(created_at, buf);
        self.preview.render(content_rect, buf);
    }
}
