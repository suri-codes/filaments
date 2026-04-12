use async_trait::async_trait;
use color_eyre::eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Size},
    widgets::ListState,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    tui::{Signal, components::Component},
    types::KastenHandle,
};

mod explorer;
use explorer::Explorer;

pub struct Todo<'text> {
    #[expect(dead_code)]
    signal_tx: Option<UnboundedSender<Signal>>,
    kh: KastenHandle,
    #[expect(dead_code)]
    layouts: Layouts,
    explorer: Explorer<'text>,
}

impl Todo<'_> {
    pub async fn new(kh: KastenHandle) -> Result<Self> {
        let kt = kh.read().await;

        let mut l_state = ListState::default();
        l_state.select_first();
        let explorer = Explorer::new(&kt.todo_tree, &kt.todo_tree.root_id, l_state, 0);

        drop(kt);

        Ok(Self {
            kh,
            layouts: Layouts::default(),
            signal_tx: None,
            explorer,
        })
    }
}

#[expect(dead_code)]
struct Layouts {
    main: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            main: Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)]),
        }
    }
}

#[async_trait]
impl Component for Todo<'_> {
    async fn init(&mut self, area: Size) -> color_eyre::Result<()> {
        let total_width = area.width;

        let mut l_state = ListState::default();
        l_state.select_first();
        let tree = &self.kh.read().await.todo_tree;

        let explorer = Explorer::new(tree, &tree.root_id, l_state, total_width);
        self.explorer = explorer;

        Ok(())
    }

    async fn update(&mut self, signal: Signal) -> color_eyre::Result<Option<Signal>> {
        match signal {
            Signal::MoveDown => {
                self.explorer.state.select_next();
                // self.update_views_from_zettel_list_selection().await?;
            }

            Signal::MoveUp => {
                self.explorer.state.select_previous();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        frame.render_stateful_widget(&self.explorer.render_list, area, &mut self.explorer.state);
        Ok(())
    }
}
