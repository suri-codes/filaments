use async_trait::async_trait;
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{Block, List, ListState},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    tui::{Signal, components::Component},
    types::{KastenHandle, ZettelId},
};

mod preview;
mod zettel_view;

use preview::Preview;
use zettel_view::ZettelView;

pub struct Zk<'text> {
    signal_tx: Option<UnboundedSender<Signal>>,
    kh: KastenHandle,
    layouts: Layouts,
    zettel_list: ZettelList<'text>,
    zettel_view: ZettelView<'text>,
    preview: Preview<'text>,
}

struct Layouts {
    left_right: Layout,
    search_zl: Layout,
    z_preview: Layout,
}

struct ZettelList<'text> {
    render_list: List<'text>,
    id_list: Vec<ZettelId>,
    state: ListState,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            left_right: Layout::horizontal(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]),
            search_zl: Layout::vertical(vec![
                Constraint::Percentage(10),
                Constraint::Percentage(90),
            ]),
            z_preview: Layout::vertical(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ]),
        }
    }
}

impl Zk<'_> {
    pub async fn new(kh: KastenHandle) -> Result<Self> {
        let kt = kh.read().await;

        let nodes = kt.graph.nodes_iter().collect::<Vec<_>>();

        let zettel_list = {
            let render_list = List::new(nodes.iter().map(|(_, n)| {
                let z = n.payload();
                let title = z.title.clone();
                let _tags = z.tags.clone();
                // let _last_modified = z.modified_at;
                Text::from(title)
            }))
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ");

            let id_list = nodes
                .iter()
                .map(|(_, n)| n.payload().id.clone())
                .collect::<Vec<_>>();

            let mut state = ListState::default();
            state.select_first();

            ZettelList {
                render_list,
                id_list,
                state,
            }
        };

        let selected_zettel = zettel_list
            .id_list
            .get(
                zettel_list
                    .state
                    .selected()
                    .expect("TODO: must handle the case where there isnt one..."),
            )
            .expect("must exist");

        let zettel = kt
            .get_by_zettel_id(selected_zettel)
            .expect("must exist, handle case where it doesnt later...")
            .payload();

        let preview = Preview::from(
            zettel
                .content(&kt.ws)
                .await
                .expect("This thing cannot be parsed properly..."),
        );

        // okay now that we have the zettel we need to construct the zettel out of this id
        let zettel_view: ZettelView = kt
            .get_by_zettel_id(selected_zettel)
            .expect("must exist, handle case where it doesnt later...")
            .payload()
            .into();

        drop(kt);

        Ok(Self {
            signal_tx: None,
            kh,
            layouts: Layouts::default(),
            zettel_list,
            zettel_view,
            preview,
        })
    }

    async fn update_views_from_zettel_list_selection(&mut self) -> Result<()> {
        let selection_idx = self
            .zettel_list
            .state
            .selected()
            .expect("i have no idea what to do if this doesnt exist");

        // sometimes the selection we get is over the length of the thing, so its
        // actually fine if this is none, just means we reached the end of the list
        let Some(z_id) = self.zettel_list.id_list.get(selection_idx) else {
            return Ok(());
        };

        let kh = self.kh.read().await;

        self.zettel_view = kh
            .get_by_zettel_id(z_id)
            .expect("this should be valid unless the kasten changed out underneath us")
            .payload()
            .into();

        self.preview = kh
            .get_by_zettel_id(z_id)
            .expect("this should be valid unless the kasten changed out underneath us")
            .payload()
            .content(&kh.ws)
            .await?
            .into();
        drop(kh);

        Ok(())
    }
}

#[async_trait]
impl Component for Zk<'_> {
    fn register_signal_handler(&mut self, tx: UnboundedSender<Signal>) -> Result<()> {
        self.signal_tx = Some(tx);
        Ok(())
    }

    async fn update(&mut self, signal: Signal) -> Result<Option<crate::tui::Signal>> {
        match signal {
            Signal::MoveDown => {
                self.zettel_list.state.select_next();
                self.update_views_from_zettel_list_selection().await?;
            }
            Signal::MoveUp => {
                self.zettel_list.state.select_previous();
                self.update_views_from_zettel_list_selection().await?;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        let (search_layout, zettel_list_layout, zettel_layout, preview_layout) = {
            let rects = self.layouts.left_right.split(area);

            let (left, right) = (rects[0], rects[1]);

            let l_rects = self.layouts.search_zl.split(left);

            let r_rects = self.layouts.z_preview.split(right);

            (l_rects[0], l_rects[1], r_rects[0], r_rects[1])
        };

        frame.render_widget(Block::new().bg(Color::Red), search_layout);

        frame.render_stateful_widget(
            &self.zettel_list.render_list,
            zettel_list_layout,
            &mut self.zettel_list.state,
        );

        frame.render_widget(self.zettel_view.clone(), zettel_layout);
        frame.render_widget(self.preview.clone(), preview_layout);

        Ok(())
    }
}
