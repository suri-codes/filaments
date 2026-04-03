use async_trait::async_trait;
use color_eyre::eyre::Result;
use dto::{QueryOrder, TagEntity, ZettelColumns, ZettelEntity};
use ratatui::{
    prelude::*,
    widgets::{Block, ListState},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    tui::{Signal, components::Component},
    types::{KastenHandle, Zettel},
};

mod preview;
mod zettel_list;
mod zettel_view;

use preview::Preview;
use zettel_list::ZettelList;
use zettel_view::ZettelView;

/// in theory we could do some fancy `type_state` encoding stuff
/// to make this work cleanly (so we know when the widgets are properly
/// initialized)
pub struct Zk<'text> {
    signal_tx: Option<UnboundedSender<Signal>>,
    // TODO: really think whether or not this actually needs a kasten
    // handle or is a workspace clone enough?
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
            z_preview: Layout::vertical(vec![Constraint::Max(6), Constraint::Percentage(80)]),
        }
    }
}

impl Zk<'_> {
    pub async fn new(kh: KastenHandle) -> Result<Self> {
        let kt = kh.read().await;

        let zettels: Vec<Zettel> = ZettelEntity::load()
            .with(TagEntity)
            .order_by_desc(ZettelColumns::ModifiedAt)
            .all(&kt.ws.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        // in theory this is wasted compute, we should be initializing all our
        // stuff inside the init function
        let mut l_state = ListState::default();
        l_state.select_first();
        let zettel_list = ZettelList::new(zettels, l_state, 0);

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
            .get_node_by_zettel_id(selected_zettel)
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
            .get_node_by_zettel_id(selected_zettel)
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
            .get_node_by_zettel_id(z_id)
            .expect("this should be valid unless the kasten changed out underneath us")
            .payload()
            .into();

        self.preview = kh
            .get_node_by_zettel_id(z_id)
            .expect("this should be valid unless the kasten changed out underneath us")
            .payload()
            .content(&kh.ws)
            .await?
            .into();
        drop(kh);

        Ok(())
    }

    pub async fn get_zettels_by_current_query(&self) -> Result<Vec<Zettel>> {
        let kt = self.kh.read().await;
        let models = ZettelEntity::load()
            .with(TagEntity)
            .order_by_desc(ZettelColumns::ModifiedAt)
            .all(&kt.ws.db)
            .await?;

        // im being a good boy and dropping this as soon as im done with the db
        drop(kt);

        let zettels: Vec<Zettel> = models.into_iter().map(Into::into).collect();
        Ok(zettels)
    }
}

#[async_trait]
impl Component for Zk<'_> {
    /// this tells us how big the space we have for this is
    async fn init(&mut self, area: Size) -> color_eyre::Result<()> {
        let total_width = area.width;

        // in theory this is wasted compute, we should be initializing all our
        let mut l_state = ListState::default();
        l_state.select_first();

        let zettel_list = ZettelList::new(
            self.get_zettels_by_current_query().await?,
            l_state,
            total_width / 2,
        );

        self.zettel_list = zettel_list;

        Ok(())
    }

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

            Signal::OpenZettel => {
                let Some(selcted) = self.zettel_list.state.selected() else {
                    return Ok(None);
                };

                let Some(zid) = self.zettel_list.id_list.get(selcted) else {
                    return Ok(None);
                };

                let kh = self.kh.read().await;
                let path = kh
                    .get_node_by_zettel_id(zid)
                    .expect(
                        "This should not have
                    change dout underneath us",
                    )
                    .payload()
                    .absolute_path(&kh.ws);

                drop(kh);

                return Ok(Some(Signal::Helix { path }));
            }

            Signal::ClosedZettel => {
                let selected = self.zettel_list.state.selected().expect(
                    "still have to
                    figure out what to do if this doesnt exist",
                );

                let Some(id) = self.zettel_list.id_list.get(selected) else {
                    return Ok(None);
                };

                let kt = self.kh.read().await;

                let node = kt
                    .get_node_by_zettel_id(id)
                    .expect("Invariant broken, this must exist.");

                // actually this is the only way to do it with the list thing,
                // the ratatui api doesnt expose a swap function to the inner render
                // list.
                self.zettel_list = ZettelList::new(
                    self.get_zettels_by_current_query().await?,
                    self.zettel_list.state,
                    self.zettel_list.width,
                );

                self.zettel_view = ZettelView::from(node.payload());
                self.preview = Preview::from(node.payload().content(&kt.ws).await?);
                drop(kt);
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
        // frame.render_widget(Block::new().bg(Color::Red), preview_layout);

        Ok(())
    }
}
