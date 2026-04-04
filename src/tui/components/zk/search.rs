use color_eyre::eyre::Error;
use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};
use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Widget},
};
use ratatui_textarea::TextArea;

use crate::types::{Workspace, Zettel};

#[derive(Clone)]
pub struct Search<'text> {
    pub query: TextArea<'text>,
    layouts: Layouts,
    matcher: Matcher,
    ws: Workspace,
}

impl Search<'_> {
    pub fn new(ws: Workspace) -> Self {
        let mut tag = TextArea::default();
        tag.set_style(Style::default());
        tag.set_block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all())
                .title("Filter by Tag"),
        );

        Self {
            matcher: Matcher::default(),
            query: Self::new_query(),
            ws,
            layouts: Layouts::default(),
        }
    }

    fn new_query<'a>() -> TextArea<'a> {
        let mut query = TextArea::default();
        query.set_style(Style::default());
        query.set_block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all())
                .title("Search"),
        );

        query.set_max_histories(0);
        query
    }

    /// Clears the query
    pub fn clear_query(&mut self) {
        self.query = Self::new_query();
    }

    /// Getter method for the current `Query`
    pub fn query(&self) -> &str {
        self.query.lines()[0].as_str()
    }

    /// Sorts the vector of `Zettels` by their relation to the
    /// search query.
    //TODO: this should really take in some sort of file cache
    // so we arent reloading it every single time...
    pub async fn rank(&mut self, zettels: Vec<Zettel>) -> Vec<Zettel> {
        // if no query, we dont do any ranking
        if self.query().is_empty() {
            return zettels;
        }

        let read_tasks = zettels
            .into_iter()
            .map(|z| {
                let ws = self.ws.clone();
                tokio::spawn(async move {
                    let content = z.content(&ws).await?;
                    let front_matter = z.front_matter(&ws).await?;
                    Ok::<(Zettel, String), Error>((z, format!("{content}\n{front_matter}")))
                })
            })
            .collect::<Vec<_>>();

        // await all of them
        let documents = futures::future::join_all(read_tasks)
            .await
            .into_iter()
            .filter_map(|result| result.ok()?.ok())
            .collect::<Vec<(Zettel, String)>>();

        let pattern = Pattern::parse(self.query(), CaseMatching::Ignore, Normalization::Smart);

        let mut results: Vec<(Zettel, u32)> = documents
            .into_iter()
            .filter_map(|(z, doc)| {
                let mut buf = Vec::new();
                let score = pattern
                    .score(Utf32Str::new(doc.as_str(), &mut buf), &mut self.matcher)
                    .unwrap_or_default();

                if score > 0 { Some((z, score)) } else { None }
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));

        results.into_iter().map(|(i, _)| i).collect()
    }
}

#[derive(Clone)]
struct Layouts {
    title: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            title: Layout::vertical(vec![Constraint::Min(3)]),
        }
    }
}

impl Widget for Search<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rect = { self.layouts.title.split(area)[0] };

        self.query.render(rect, buf);
    }
}
