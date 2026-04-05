use async_trait::async_trait;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use dto::{QueryOrder, TagEntity, ZettelColumns, ZettelEntity};
use ratatui::{prelude::*, widgets::ListState};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use crate::{
    tui::{Signal, components::Component},
    types::{KastenHandle, Zettel},
};

mod preview;
mod search;
mod zettel_list;
mod zettel_view;

use preview::Preview;
use search::Search;
use zettel_list::ZettelList;
use zettel_view::ZettelView;

/// in theory we could do some fancy `type_state` encoding stuff
/// to make this work cleanly (so we know when the widgets are properly
/// initialized)
/// The tui interface for interacting with a `ZettelKasten`.
/// Has `Search` functionality and `Preview` to view each `Zettel`.
pub struct Zk<'text> {
    signal_tx: Option<UnboundedSender<Signal>>,
    kh: KastenHandle,
    layouts: Layouts,
    search: Search<'text>,
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
                Constraint::Fill(51),
                Constraint::Min(1),
                Constraint::Fill(50),
            ]),
            search_zl: Layout::vertical(vec![Constraint::Min(3), Constraint::Fill(95)]),
            z_preview: Layout::vertical(vec![Constraint::Min(6), Constraint::Fill(95)]),
        }
    }
}

impl Zk<'_> {
    pub async fn new(kh: KastenHandle) -> Result<Self> {
        let kt = kh.read().await;
        let ws = kt.ws.clone();

        let fetch_all = async || -> Result<Vec<Zettel>> {
            Ok(ZettelEntity::load()
                .with(TagEntity)
                .order_by_desc(ZettelColumns::ModifiedAt)
                .all(&ws.db)
                .await?
                .into_iter()
                .map(Into::into)
                .collect())
        };

        let mut zettels: Vec<Zettel> = fetch_all().await?;

        drop(kt);
        if zettels.is_empty() {
            let z = Zettel::new("Welcome!", &ws).await?;
            kh.write().await.process_path(&z.absolute_path(&ws)).await?;
            zettels = fetch_all().await?;
        }

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
                    .expect("We explicitly select the first item"),
            )
            // so technically this might not exist
            .expect("There must always be one atleast one zettel");

        let kt = kh.read().await;

        info!("{selected_zettel:#?}");
        info!("{kt:#?}");

        let zettel = kt
            .get_node_by_zettel_id(selected_zettel)
            .expect("kasten should have the selected zettel")
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

        let ws = kt.ws.clone();

        drop(kt);

        Ok(Self {
            signal_tx: None,
            kh,
            layouts: Layouts::default(),
            zettel_list,
            zettel_view,
            preview,
            search: Search::new(ws),
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

        // for now we are going to just read that shit every time...

        let zettels: Vec<Zettel> = models.into_iter().map(Into::into).collect();

        Ok(zettels)
    }

    pub async fn update_with_respect_to_query(&mut self) -> Result<()> {
        let zettels = self
            .search
            .rank(self.get_zettels_by_current_query().await?)
            .await;

        self.zettel_list = ZettelList::new(zettels, self.zettel_list.state, self.zettel_list.width);
        info!("we are moving selection to first");
        self.zettel_list.state.select_first();
        self.update_views_from_zettel_list_selection().await?;

        Ok(())
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

            Signal::NewZettel => {
                // what the fuck am i going to do in here

                let ws = &self.kh.read().await.ws;

                // we create the zettel with the query as the
                let z = Zettel::new(self.search.query(), ws).await?;

                let path = z.absolute_path(ws);

                return Ok(Some(Signal::Helix { path }));
            }

            Signal::ClosedZettel => {
                let selected = self
                    .zettel_list
                    .state
                    .selected()
                    .expect("This must be the zettel we just edited");

                let Some(id) = self.zettel_list.id_list.get(selected) else {
                    return Ok(None);
                };

                let kt = self.kh.read().await;

                let node = kt
                    .get_node_by_zettel_id(id)
                    .expect("Invariant broken, this must exist.");

                // reset the state of the component
                self.search.clear_query();
                self.zettel_list.state.select_first();

                // regenerate a fresh zettel list
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

    async fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Signal>> {
        // NOTE: this is hardcoded for now, but I honestly think people should not
        // be able to change these binds, opinionated software or something...
        if !(key.code.is_up() || key.code.is_down() || key.code.is_enter()) {
            self.search.query.input(key);
            self.update_with_respect_to_query().await?;
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

            let (left, right) = (rects[0], rects[2]);

            let l_rects = self.layouts.search_zl.split(left);

            let r_rects = self.layouts.z_preview.split(right);

            (l_rects[0], l_rects[1], r_rects[0], r_rects[1])
        };

        frame.render_widget(self.search.clone(), search_layout);

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
