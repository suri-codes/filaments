use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListState},
};
use tracing::info;
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
        .highlight_style(Modifier::ITALIC | Modifier::BOLD)
        .highlight_symbol("> ");

        let id_list = tree
            .tree
            .traverse_pre_order_ids(scope)
            .expect("This should not panic as the node id should exist inside")
            .collect::<Vec<_>>();

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
                .title("[1]")
                .title("Explorer")
                .borders(Borders::TOP | Borders::RIGHT)
                .border_style(Style::new().fg(Color::Green))
                .border_type(BorderType::Rounded),
        );
    }

    pub fn set_inactive(&mut self) {
        self.render_list = self.render_list.clone().block(
            Block::new()
                .title("[1]")
                .title("Explorer")
                .borders(Borders::TOP | Borders::RIGHT)
                .border_style(Style::new().fg(Color::Gray))
                .border_type(BorderType::Rounded),
        );
    }
}

pub struct ExplorerListItem<'text> {
    spacer: Span<'text>,
    name: Span<'text>,
    width: u16,
}

impl From<&TodoNode> for ExplorerListItem<'_> {
    fn from(value: &TodoNode) -> Self {
        let spacer = Span::from("  ".repeat(value.depth));
        let name = match value.kind {
            TodoNodeKind::Group(ref g) => Span::from(format!(" {}", g.name.clone()))
                .bg(g.tag.color)
                .fg(Color::Black),
            TodoNodeKind::Task(ref t) => {
                Span::from(format!(" {}", t.name.clone())).fg(t.group.tag.color)
            }
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
        let bg_color = value.name.style.bg;
        let spacer_width = value.spacer.content.len();
        let name_width = value.name.content.len();
        let used = spacer_width + name_width;

        let mut spans = vec![value.spacer, value.name];

        if let Some(color) = bg_color {
            let padding = (value.width as usize).saturating_sub(used);
            spans.push(Span::styled(
                " ".repeat(padding),
                Style::default().bg(color),
            ));
        }

        info!("{spans:#?}");

        Line::from(spans).into()
    }
}
