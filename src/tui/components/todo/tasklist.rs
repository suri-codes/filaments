#![expect(dead_code)]
use ratatui::{
    style::{Color, Modifier},
    text::{Line, Span, Text},
    widgets::{List, ListState},
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
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ");

        Self {
            render_list,
            id_list,
            state,
            width,
        }
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

        let name = Span::from(task.name.clone());
        let group = Span::from(task.group.name.clone());
        let due_priority = task.due.map_or_else(
            || Span::from(task.priority.to_string()),
            |due_date| Span::from(due_date.to_string()),
        );

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
        let line = Line::from(vec![value.name, value.group, value.due_priority]);
        line.into()
    }
}
