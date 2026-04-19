use async_trait::async_trait;
use color_eyre::eyre::{Context as _, ContextCompat, Result};
use crossterm::event::KeyEvent;
use dto::{QueryOrder, TagEntity, ZettelColumns, ZettelEntity};
use ratatui::{prelude::*, widgets::ListState};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use crate::{
    tui::{Page, Signal, components::Component},
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
    area: Size,

    search: Option<Search<'text>>,
    zettel_list: Option<ZettelList<'text>>,
    zettel_view: Option<ZettelView<'text>>,
    preview: Option<Preview<'text>>,

    active: bool,
}

struct Layouts {
    left_right: Layout,
    search_zl: Layout,
    z_preview: Layout,
}

impl Layouts {
    fn split(&self, area: Rect) -> LayoutSplit {
        let rects = self.left_right.split(area);

        let (left, right) = (rects[0], rects[2]);

        let l_rects = self.search_zl.split(left);

        let r_rects = self.z_preview.split(right);

        LayoutSplit {
            search: l_rects[0],
            zettel_list: l_rects[1],
            zettel_view: r_rects[0],
            preview: r_rects[1],
        }
    }
}

struct LayoutSplit {
    search: Rect,
    zettel_list: Rect,
    zettel_view: Rect,
    preview: Rect,
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
    pub fn new(kh: KastenHandle) -> Self {
        Self {
            signal_tx: None,
            kh,

            layouts: Layouts::default(),
            area: Size::default(),

            search: None,
            zettel_list: None,
            zettel_view: None,
            preview: None,

            active: false,
        }
    }

    async fn refresh(&mut self) -> Result<()> {
        let zettel_list = self.zettel_list.as_mut().expect("Must be initialized");

        let selected = zettel_list
            .state
            .selected()
            .and_then(|idx| zettel_list.id_list.get(idx));

        let splits = self
            .layouts
            .split(Rect::new(0, 0, self.area.width, self.area.height));

        let kt = self.kh.read().await;
        let db = kt.db.clone();

        // ideally we just keep the same selection as we had originally

        let fetch_all = async || -> Result<Vec<Zettel>> {
            Ok(ZettelEntity::load()
                .with(TagEntity)
                .order_by_desc(ZettelColumns::ModifiedAt)
                .all(&db)
                .await?
                .into_iter()
                .map(Into::into)
                .collect())
        };

        let zettels: Vec<Zettel> = fetch_all().await?;

        let mut zettel_list = ZettelList::new(
            zettels.clone(),
            ListState::default(),
            splits.zettel_list.width,
        );

        let selected_zettel_idx =
            selected.and_then(|desired| zettel_list.id_list.iter().position(|id| id == desired));

        zettel_list.state.select(selected_zettel_idx);

        let zettel = zettels
            .iter()
            //TODO: expect probably should not look like this
            .find(|&z| &z.id == selected.expect("Something should be selected"))
            .expect("we selected it out of the list so it must exist");

        let preview = Preview::from(zettel.content(&kt.index).clone());

        drop(kt);

        let search = Search::new(self.kh.clone());
        let zettel_view = zettel.into();

        self.search = Some(search);
        self.zettel_view = Some(zettel_view);
        self.preview = Some(preview);
        self.zettel_list = Some(zettel_list);

        Ok(())
    }

    async fn update_views_from_zettel_list_selection(&mut self) -> Result<()> {
        let zettel_list = self.zettel_list.as_mut().expect("Must be initialzied");

        let selection_idx = zettel_list
            .state
            .selected()
            .expect("i have no idea what to do if this doesnt exist");

        // sometimes the selection we get is over the length of the thing, so its
        // actually fine if this is none, just means we reached the end of the list
        let Some(zid) = zettel_list.id_list.get(selection_idx) else {
            return Ok(());
        };

        let kh = self.kh.read().await;

        let zettel = &Zettel::fetch_from_db(zid, &kh.db)
            .await?
            .context("Unknown Behaviour, A selected zettel got deleted somehow.")?;

        self.preview = Some(zettel.content(&kh.index).clone().into());
        drop(kh);

        self.zettel_view = Some(zettel.into());

        Ok(())
    }

    pub async fn get_zettels_by_current_query(&self) -> Result<Vec<Zettel>> {
        let kt = self.kh.read().await;
        let models = ZettelEntity::load()
            .with(TagEntity)
            .order_by_desc(ZettelColumns::ModifiedAt)
            .all(&kt.db)
            .await?;

        // im being a good boy and dropping this as soon as im done with the db
        drop(kt);

        // for now we are going to just read that shit every time...

        let zettels: Vec<Zettel> = models.into_iter().map(Into::into).collect();

        Ok(zettels)
    }

    pub async fn update_with_respect_to_query(&mut self) -> Result<()> {
        let curr_zettels = self.get_zettels_by_current_query().await?;

        let search = self
            .search
            .as_mut()
            .expect("Must be initalized by this point");

        let zettel_list = self
            .zettel_list
            .as_mut()
            .expect("Must be initialized by this point");

        let zettels = search.rank(curr_zettels).await;

        *zettel_list = ZettelList::new(zettels, zettel_list.state, zettel_list.width);

        info!("we are moving selection to first");

        zettel_list.state.select_first();
        self.update_views_from_zettel_list_selection().await?;

        Ok(())
    }
}

#[async_trait]
impl Component for Zk<'_> {
    /// this tells us how big the space we have for this is
    async fn init(&mut self, area: Size) -> color_eyre::Result<()> {
        self.area = area;
        let splits = self.layouts.split(Rect::new(0, 0, area.width, area.height));
        let mut kt = self.kh.write().await;
        let db = kt.db.clone();

        let fetch_all = async || -> Result<Vec<Zettel>> {
            Ok(ZettelEntity::load()
                .with(TagEntity)
                .order_by_desc(ZettelColumns::ModifiedAt)
                .all(&db)
                .await?
                .into_iter()
                .map(Into::into)
                .collect())
        };

        let mut zettels: Vec<Zettel> = fetch_all().await?;

        if zettels.is_empty() {
            let _ = Zettel::new("Welcome!", &mut kt, vec![]).await?;
            zettels = fetch_all().await?;
        }

        // in theory this is wasted compute, we should be initializing all our
        // stuff inside the init function
        let mut l_state = ListState::default();
        l_state.select_first();
        let zettel_list = ZettelList::new(zettels.clone(), l_state, splits.zettel_list.width);

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

        let zettel = zettels
            .iter()
            .find(|&z| &z.id == selected_zettel)
            .expect("we selected it out of the list so it must exist");

        let preview = Preview::from(zettel.content(&kt.index).clone());

        drop(kt);

        let search = Search::new(self.kh.clone());
        let zettel_view = zettel.into();

        self.search = Some(search);
        self.zettel_view = Some(zettel_view);
        self.preview = Some(preview);
        self.zettel_list = Some(zettel_list);

        Ok(())
    }

    fn register_signal_handler(&mut self, tx: UnboundedSender<Signal>) -> Result<()> {
        self.signal_tx = Some(tx);
        Ok(())
    }

    async fn update(&mut self, signal: Signal) -> Result<Option<crate::tui::Signal>> {
        let zettel_list = self.zettel_list.as_mut().expect("Must be initialized");
        let search = self.search.as_mut().expect("Must be initialized");
        match signal {
            Signal::SwitchTo { page } => {
                self.active = page == Page::Zk;
            }

            Signal::Refresh => {
                self.refresh().await?;
            }

            Signal::MoveDown if self.active => {
                zettel_list.state.select_next();
                self.update_views_from_zettel_list_selection().await?;
            }
            Signal::MoveUp if self.active => {
                zettel_list.state.select_previous();
                self.update_views_from_zettel_list_selection().await?;
            }

            Signal::OpenZettel if self.active => {
                let Some(selcted) = zettel_list.state.selected() else {
                    return Ok(None);
                };

                let Some(zid) = zettel_list.id_list.get(selcted) else {
                    return Ok(None);
                };

                let kh = self.kh.read().await;

                let path = kh.index.get_zod(zid).path.clone();

                drop(kh);

                return Ok(Some(Signal::Helix { path }));
            }

            Signal::NewZettel => {
                // what the fuck am i going to do in here

                let mut kt = self.kh.write().await;

                // we create the zettel with the query as the
                let z = Zettel::new(search.query(), &mut kt, vec![])
                    .await
                    .with_context(|| "Failed to create a new Zettel!")?;

                // let path = z.absolute_path(&kt.index).to_path_buf();

                drop(kt);

                return Ok(Some(Signal::CreatedZettel { zid: z.id }));

                // return Ok(Some(Signal::Helix { path }));
            }
            Signal::CreatedZettel { zid } => {
                // what the fuck am i going to do in here

                let kt = self.kh.read().await;

                let path = kt.index.get_zod(&zid).path.clone();

                // let path = z.absolute_path(&kt.index).to_path_buf();

                drop(kt);

                return Ok(Some(Signal::Helix { path }));
            }

            Signal::ClosedZettel { zid } => {
                let curr_zettels = self.get_zettels_by_current_query().await?; // regenerate a fresh zettel list

                let zettel_list = self.zettel_list.as_mut().expect("Must be initialized");
                let zettel_view = self.zettel_view.as_mut().expect("Must be initialized");
                let preview = self.preview.as_mut().expect("Must be initialized");
                let search = self.search.as_mut().expect("Must be initialized");

                *zettel_list = ZettelList::new(curr_zettels, zettel_list.state, zettel_list.width);

                let kt = self.kh.read().await;

                let zettel = Zettel::fetch_from_db(&zid, &kt.db)
                    .await?
                    .expect("invariant broken, we just closed this zettel");

                let idx = zettel_list.id_list.iter().position(|id| *id == zettel.id);

                // reset the state of the component
                search.clear_query();
                zettel_list.state.select(idx);

                *zettel_view = ZettelView::from(&zettel);
                *preview = Preview::from(zettel.content(&kt.index).clone());
                drop(kt);
            }

            _ => {}
        }
        Ok(None)
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Signal>> {
        // NOTE: this is hardcoded for now, but I honestly think people should not
        // be able to change these binds, opinionated software or something...
        if !(key.code.is_up() || key.code.is_down() || key.code.is_enter() || key.code.is_tab()) {
            self.search.as_mut().unwrap().query.input(key);
            self.update_with_respect_to_query().await?;
        }

        Ok(None)
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        let zettel_list = self.zettel_list.as_mut().expect("Must be initialized");
        let zettel_view = self.zettel_view.as_mut().expect("Must be initialized");
        let preview = self.preview.as_mut().expect("Must be initialized");
        let search = self.search.as_mut().expect("Must be initialized");

        let splits = self.layouts.split(area);

        frame.render_widget(search.clone(), splits.search);

        frame.render_stateful_widget(
            &zettel_list.render_list,
            splits.zettel_list,
            &mut zettel_list.state,
        );

        frame.render_widget(zettel_view.clone(), splits.zettel_view);
        frame.render_widget(preview.clone(), splits.preview);

        Ok(())
    }
}
