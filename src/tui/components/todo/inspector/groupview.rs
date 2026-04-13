use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Paragraph, Widget},
};

use crate::types::Group;

#[derive(Debug, Clone)]
pub struct GroupView<'text> {
    name: Paragraph<'text>,
    priority: Paragraph<'text>,
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
                Constraint::Percentage(50),
                Constraint::Fill(100),
            ]),
            name_priority_created_at: Layout::vertical(vec![
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]),
        }
    }
}

impl From<&Group> for GroupView<'_> {
    fn from(value: &Group) -> Self {
        Self {
            name: Paragraph::new(value.name.clone()),
            priority: Paragraph::new(value.priority.to_string()),
            created_at: Paragraph::new(value.created_at()),
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
