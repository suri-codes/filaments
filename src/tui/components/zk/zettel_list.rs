use egui_graphs::{Node, petgraph::graph::NodeIndex};
use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListState},
};

use crate::types::{Link, Zettel, ZettelId};

pub struct ZettelList<'text> {
    pub render_list: ratatui::widgets::List<'text>,
    pub id_list: Vec<ZettelId>,
    pub state: ListState,
    pub width: u16,
}

pub struct ZettelListItem<'text> {
    title: Span<'text>,
    tags: Vec<Span<'text>>,
    date: Span<'text>,
    width: u16,
}

impl From<&Zettel> for ZettelListItem<'_> {
    fn from(value: &Zettel) -> Self {
        Self {
            title: Span::from(value.title.clone())
                .style(Style::new().cyan())
                .add_modifier(Modifier::BOLD),
            tags: value
                .tags
                .iter()
                .map(|t| {
                    Span::from(format!("{} ", t.name))
                        .style(Style::new().fg(t.color.into()).italic())
                        .add_modifier(Modifier::DIM)
                })
                .collect(),
            date: Span::from(value.created_at()).style(Style::new().add_modifier(Modifier::DIM)),
            width: 0,
        }
    }
}

impl<'item, 'text> From<ZettelListItem<'item>> for Text<'text>
where
    'item: 'text,
{
    fn from(value: ZettelListItem<'item>) -> Self {
        let title_width: usize = value.title.width();
        let tags_width: usize = value.tags.iter().map(Span::width).sum();
        let date_width: usize = value.date.width() + 2;

        // tags start 2 tabs after the title
        let gap_after_title = 2;
        let used = title_width + gap_after_title + tags_width + date_width;
        let padding = (value.width as usize).saturating_sub(used);

        let mut spans: Vec<Span> = std::iter::once(value.title)
            .chain(std::iter::once(Span::raw("  ")))
            .chain(value.tags)
            .chain(std::iter::once(Span::raw(" ".repeat(padding))))
            .collect();

        if tags_width + title_width + date_width <= value.width as usize {
            spans.push(value.date);
        }

        let line = Line::from(spans);
        line.into()
    }
}

impl ZettelList<'_> {
    pub fn new(nodes: &[(NodeIndex, &Node<Zettel, Link>)], state: ListState, width: u16) -> Self {
        let render_list = List::new(nodes.iter().map(|(_, n)| {
            let z = n.payload();
            let mut zli: ZettelListItem<'_> = z.into();
            zli.width = width;

            Text::from(zli)
        }))
        .style(Color::White)
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ");

        let id_list = nodes
            .iter()
            .map(|(_, n)| n.payload().id.clone())
            .collect::<Vec<_>>();

        ZettelList {
            render_list,
            id_list,
            state,
            width,
        }
    }
}
