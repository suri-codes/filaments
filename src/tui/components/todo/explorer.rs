use ratatui::{
    style::{Color, Modifier},
    text::{Line, Span, Text},
    widgets::{List, ListState},
};
use tree::NodeId;

use crate::types::{TodoNode, TodoNodeKind, TodoTree};

pub struct Explorer<'text> {
    pub render_list: ratatui::widgets::List<'text>,
    #[allow(dead_code)]
    pub id_list: Vec<NodeId>,
    pub state: ListState,
    #[allow(dead_code)]
    pub width: u16,
}

impl Explorer<'_> {
    pub fn new(tree: &TodoTree, scope: &NodeId, state: ListState, width: u16) -> Self {
        let render_list = List::new(
            tree.tree
                .traverse_pre_order(scope)
                .expect("This should not panic as the node id should exist inside")
                .filter_map(|node| {
                    // we dont want to show the root
                    if node.data().kind == TodoNodeKind::Root {
                        return None;
                    }
                    let mut eli: ExplorerListItem<'_> = node.data().into();
                    eli.width = width;
                    Some(Text::from(eli))
                }),
        )
        .style(Color::White)
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ");

        let id_list = tree
            .tree
            .traverse_pre_order_ids(scope)
            .expect("This should nto panic as the node id should exist inside")
            .collect::<Vec<_>>();

        Self {
            render_list,
            id_list,
            state,
            width,
        }
    }
}

pub struct ExplorerListItem<'text> {
    spacer: Span<'text>,
    name: Span<'text>,
    width: u16,
}

impl From<&TodoNode> for ExplorerListItem<'_> {
    fn from(value: &TodoNode) -> Self {
        let spacer = Span::from(" ".repeat(value.depth));
        let name = match value.kind {
            TodoNodeKind::Group(ref g) => Span::from(g.name.clone()),
            TodoNodeKind::Task(ref t) => Span::from(t.name.clone()),
            TodoNodeKind::Root => Span::from("THIS SHOULD NOT BE VISIBLE"),
        };

        Self {
            spacer,
            name,
            width: 0,
        }
    }
}

impl<'text> From<ExplorerListItem<'text>> for Text<'text> {
    fn from(value: ExplorerListItem<'text>) -> Self {
        let line = Line::from(vec![value.spacer, value.name]);
        line.into()
    }
}
