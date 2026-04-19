use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, Paragraph, Widget},
};
use ratatui_textarea::TextArea;

use crate::types::Task;

#[derive(Debug, Clone)]
pub struct TaskView<'text> {
    pub name: TextArea<'text>,
    pub priority: TextArea<'text>,
    pub due_finished_at: TextArea<'text>,
    parent_group: Paragraph<'text>,
    layouts: Layouts,
}

#[derive(Debug, Clone)]
struct Layouts {
    left_content: Layout,
    name_priority_due_group: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            left_content: Layout::horizontal(vec![
                Constraint::Percentage(30),
                Constraint::Fill(100),
            ]),
            name_priority_due_group: Layout::vertical(vec![
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Min(3),
            ]),
        }
    }
}

impl From<&Task> for TaskView<'_> {
    fn from(value: &Task) -> Self {
        let mut name = TextArea::new(vec![value.name.clone()]);
        name.set_block(Block::bordered().title("[N]ame"));
        name.set_cursor_style(Style::reset());
        name.set_cursor_line_style(Style::reset());

        let mut priority = TextArea::new(vec![value.priority.to_string()]);
        priority.set_block(Block::bordered().title("[P]riority"));
        priority.set_cursor_style(Style::reset());
        priority.set_cursor_line_style(Style::reset());

        let due_finished_at = {
            let (title, content) = value.finished_at().map_or_else(
                || ("[D]ue", value.due.to_string()),
                |finished| ("[F]inished At", finished),
            );

            let mut textarea = TextArea::new(vec![content]);
            textarea.set_block(Block::bordered().title(title));
            textarea.set_cursor_style(Style::reset());
            textarea.set_cursor_line_style(Style::reset());
            textarea
        };
        Self {
            name,
            priority,
            due_finished_at,
            parent_group: Paragraph::new(value.group.name.clone())
                .block(Block::bordered().title("Group")),
            layouts: Layouts::default(),
        }
    }
}

impl Widget for TaskView<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (name_rect, priority_rect, due_rect, group_rect, _content_rect) = {
            let rects = self.layouts.left_content.split(area);
            let l_rects = self.layouts.name_priority_due_group.split(rects[0]);

            (l_rects[0], l_rects[1], l_rects[2], l_rects[3], rects[1])
        };

        self.name.render(name_rect, buf);
        self.priority.render(priority_rect, buf);
        self.due_finished_at.render(due_rect, buf);
        self.parent_group.render(group_rect, buf);
    }
}
