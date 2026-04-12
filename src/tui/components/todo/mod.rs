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
mod tasklist;
use tasklist::TaskList;

pub struct Todo<'text> {
    #[expect(dead_code)]
    signal_tx: Option<UnboundedSender<Signal>>,
    kh: KastenHandle,
    layouts: Layouts,
    explorer: Explorer<'text>,
    task_list: TaskList<'text>,
}

impl Todo<'_> {
    pub async fn new(kh: KastenHandle) -> Result<Self> {
        let kt = kh.read().await;

        let mut l_state = ListState::default();
        l_state.select_first();
        let explorer = Explorer::new(&kt.todo_tree, &kt.todo_tree.root_id, l_state, 0);
        let task_list = TaskList::new(&kt.todo_tree, &kt.todo_tree.root_id, l_state, 0);

        drop(kt);

        Ok(Self {
            kh,
            layouts: Layouts::default(),
            signal_tx: None,
            explorer,
            task_list,
        })
    }
}

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
        let tree = &self.kh.read().await.todo_tree;

        let explorer = Explorer::new(tree, &tree.root_id, self.explorer.state, total_width / 2);
        let task_list = TaskList::new(tree, &tree.root_id, self.task_list.state, total_width / 2);
        self.explorer = explorer;
        self.task_list = task_list;

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
        let (explorer_rect, task_list_rect) = {
            let rects = self.layouts.main.split(area);
            (rects[0], rects[1])
        };

        frame.render_stateful_widget(
            &self.explorer.render_list,
            explorer_rect,
            &mut self.explorer.state,
        );

        frame.render_stateful_widget(
            &self.task_list.render_list,
            task_list_rect,
            &mut self.task_list.state,
        );
        Ok(())
    }
}
