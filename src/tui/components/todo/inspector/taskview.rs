use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Paragraph, Widget},
};

use crate::types::Task;

#[derive(Debug, Clone)]
pub struct TaskView<'text> {
    name: Paragraph<'text>,
    priority: Paragraph<'text>,
    #[expect(dead_code)]
    created_at: Paragraph<'text>,
    #[expect(dead_code)]
    parent_group: Paragraph<'text>,

    due: Paragraph<'text>,

    layouts: Layouts,
}

#[derive(Debug, Clone)]
struct Layouts {
    left_content: Layout,
    name_priority_due: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            left_content: Layout::horizontal(vec![
                Constraint::Percentage(50),
                Constraint::Fill(100),
            ]),
            name_priority_due: Layout::vertical(vec![
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]),
        }
    }
}

impl From<&Task> for TaskView<'_> {
    fn from(value: &Task) -> Self {
        Self {
            name: Paragraph::new(value.name.clone()),
            priority: Paragraph::new(value.priority.to_string()),
            created_at: Paragraph::new(value.created_at()),
            parent_group: Paragraph::new(value.group.name.clone()),
            due: Paragraph::new(value.due().unwrap_or_else(|| "None".to_owned())),
            layouts: Layouts::default(),
        }
    }
}

impl Widget for TaskView<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (name_rect, priority_rect, due_rect, _content_rect) = {
            let rects = self.layouts.left_content.split(area);
            let l_rects = self.layouts.name_priority_due.split(rects[0]);

            (l_rects[0], l_rects[1], l_rects[2], rects[1])
        };

        self.name.render(name_rect, buf);
        self.priority.render(priority_rect, buf);
        self.due.render(due_rect, buf);
    }
}
