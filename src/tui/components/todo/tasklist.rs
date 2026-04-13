#![expect(dead_code)]
use ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListState},
};
use tree::NodeId;

use crate::types::{TodoNode, TodoNodeKind, TodoTree};

pub struct TaskList<'text> {
    pub render_list: List<'text>,
    pub id_list: Vec<NodeId>,
    pub state: ListState,
    pub width: u16,
}

impl TaskList<'_> {
    pub fn new(tree: &TodoTree, scope: &NodeId, state: ListState, width: u16) -> Self {
        let mut id_list = vec![];

        let render_list = List::new(
            tree.tree
                .traverse_pre_order(scope)
                .expect("nthis should not panic as the node id should exist inside")
                .zip(
                    tree.tree
                        .traverse_pre_order_ids(scope)
                        .expect("This should not panic as the nodeid should exist inside"),
                )
                .filter_map(|(node, id)| {
                    let TodoNodeKind::Task(_) = node.data().kind else {
                        return None;
                    };

                    let mut tli: TaskListItem<'_> = node.data().into();

                    id_list.push(id);

                    tli.width = width;
                    Some(Text::from(tli))
                }),
        )
        .style(Color::White)
        .highlight_style(Style::new().on_dark_gray());
        // .highlight_symbol("> ");

        Self {
            render_list,
            id_list,
            state,
            width,
        }
    }

    pub fn set_active(&mut self) {
        self.render_list = self.render_list.clone().block(
            Block::new()
                .title("[3]")
                .title("TaskList")
                .borders(Borders::TOP | Borders::LEFT)
                .border_style(Style::new().fg(Color::Green))
                .border_type(BorderType::Rounded),
        );
    }

    pub fn set_inactive(&mut self) {
        self.render_list = self.render_list.clone().block(
            Block::new()
                .title("[3]")
                .title("TaskList")
                .borders(Borders::TOP | Borders::LEFT)
                .border_style(Style::new().fg(Color::Gray))
                .border_type(BorderType::Rounded),
        );
    }
}

pub struct TaskListItem<'text> {
    name: Span<'text>,
    group: Span<'text>,
    due_priority: Span<'text>,
    width: u16,
}

// its fine because if it fails, its my fault, not the users.
#[expect(clippy::fallible_impl_from)]
impl From<&TodoNode> for TaskListItem<'_> {
    fn from(value: &TodoNode) -> Self {
        let TodoNodeKind::Task(ref task) = value.kind else {
            panic!("Should not be possible");
        };

        let color = task.group.tag.color;

        let name = Span::from(task.name.clone()).style(Style::new().fg(color.into()));
        let group = Span::from(task.group.name.clone()).style(Style::new().fg(color.into()));
        let due_priority = task
            .due()
            .map_or_else(|| Span::from(task.priority.to_string()), Span::from)
            .style(Style::new().fg(color.into()));

        Self {
            name,
            group,
            due_priority,
            width: 0,
        }
    }
}

impl<'text> From<TaskListItem<'text>> for Text<'text> {
    fn from(value: TaskListItem<'text>) -> Self {
        let total_width = value.width.saturating_sub(2) as usize;
        let name_col = total_width / 2;
        let due_content = value.due_priority.content.as_ref();
        let due_col = due_content.len();
        let group_col = total_width.saturating_sub(name_col + due_col);

        let name_str = format!("{:<width$}", value.name.content, width = name_col);
        let group_str = format!("{:<width$}", value.group.content, width = group_col);
        let due_str = format!("{due_content:>due_col$}");

        let name = Span::styled(name_str, value.name.style);
        let group = Span::styled(group_str, value.group.style);
        let due = Span::styled(due_str, value.due_priority.style);

        Line::from(vec![name, group, due]).into()
    }
}
