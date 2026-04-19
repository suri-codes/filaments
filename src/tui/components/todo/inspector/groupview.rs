use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, Paragraph, Widget},
};
use ratatui_textarea::TextArea;

use crate::types::Group;

#[derive(Debug, Clone)]
pub struct GroupView<'text> {
    pub name: TextArea<'text>,
    pub priority: TextArea<'text>,
    created_at: Paragraph<'text>,
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

impl From<&Group> for GroupView<'_> {
    fn from(value: &Group) -> Self {
        let mut name = TextArea::new(vec![value.name.clone()]);
        name.set_block(Block::bordered().title("[N]ame"));
        name.set_cursor_style(Style::reset());
        name.set_cursor_line_style(Style::reset());

        let mut priority = TextArea::new(vec![value.priority.to_string()]);
        priority.set_block(Block::bordered().title("[P]riority"));
        priority.set_cursor_style(Style::reset());
        priority.set_cursor_line_style(Style::reset());

        Self {
            name,
            priority,
            created_at: Paragraph::new(value.created_at())
                .block(Block::bordered().title("Created At")),
            layouts: Layouts::default(),
        }
    }
}

impl Widget for GroupView<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (name_rect, priority_rect, created_at, _content_rect) = {
            let rects = self.layouts.left_content.split(area);
            let l_rects = self.layouts.name_priority_created_at.split(rects[0]);

            (l_rects[0], l_rects[1], l_rects[2], rects[1])
        };

        self.name.render(name_rect, buf);
        self.priority.render(priority_rect, buf);
        self.created_at.render(created_at, buf);
    }
}
