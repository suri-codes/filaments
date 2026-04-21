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

        let mut items = tree
            .tree
            .traverse_pre_order(scope)
            .expect("This should not panic as the node id sohuld exist inside")
            .zip(
                tree.tree
                    .traverse_pre_order_ids(scope)
                    .expect("This should not panic as the nodeid should exist inside"),
            )
            .filter(|(node, _)| {
                if let TodoNodeKind::Task(ref t) = node.data().kind
                    && t.finished_at().is_none()
                {
                    return true;
                }
                false
            })
            .collect::<Vec<_>>();

        items.sort_by(|(a, _), (b, _)| a.data().p_score.total_cmp(&b.data().p_score));

        items.reverse();

        let render_list = List::new(items.into_iter().map(|(node, id)| {
            let TodoNodeKind::Task(_) = node.data().kind else {
                unreachable!("we already filtered for this earlier")
            };

            let mut tli: TaskListItem<'_> = node.data().into();

            id_list.push(id);

            tli.width = width;
            Text::from(tli)
        }))
        .style(Color::White)
        .highlight_style(Style::new().on_dark_gray());

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
    p_score: Span<'text>,
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
        let due_priority = Span::from(if task.due.has_date() {
            task.due.to_string()
        } else {
            task.priority.to_string()
        })
        .style(Style::new().fg(color.into()));

        let p_score =
            Span::from(format!("{:.3}", value.p_score)).style(Style::new().fg(color.into()));

        Self {
            name,
            group,
            due_priority,
            width: 0,
            p_score,
        }
    }
}

impl<'text> From<TaskListItem<'text>> for Text<'text> {
    fn from(value: TaskListItem<'text>) -> Self {
        let total_width = value.width.saturating_sub(2) as usize;
        let name_col = 4 * total_width / 9;
        let p_score_col = 10; // e.g. "0.103" — fixed width
        let due_col = 22; // enough for "2026-04-22 11:59:59 PM" or a priority label
        let group_col = total_width.saturating_sub(name_col + p_score_col + due_col);

        let name_str = if value.name.content.len() > name_col {
            let truncated: String = value
                .name
                .content
                .chars()
                .take(name_col.saturating_sub(5))
                .collect();
            format!("{truncated}...  ")
        } else {
            format!("{:<width$}", value.name.content, width = name_col)
        };

        let group_str = format!("{:<width$}", value.group.content, width = group_col);
        let p_score_str = format!("{:<width$}", value.p_score.content, width = p_score_col);
        let due_str = format!("{:>width$}", value.due_priority.content, width = due_col);

        let name = Span::styled(name_str, value.name.style);
        let group = Span::styled(group_str, value.group.style);
        let p_score = Span::styled(p_score_str, value.p_score.style);
        let due = Span::styled(due_str, value.due_priority.style);

        Line::from(vec![name, group, p_score, due]).into()
    }
}
