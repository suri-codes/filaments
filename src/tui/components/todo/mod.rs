use async_trait::async_trait;
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
    explorer: Option<Explorer<'text>>,
    task_list: Option<TaskList<'text>>,
}

impl Todo<'_> {
    pub fn new(kh: KastenHandle) -> Self {
        Self {
            kh,
            layouts: Layouts::default(),
            signal_tx: None,
            explorer: None,
            task_list: None,
        }
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

        let mut l_state = ListState::default();

        l_state.select_first();

        let mut explorer = Explorer::new(tree, &tree.root_id, l_state, total_width / 2);
        let task_list = TaskList::new(tree, &tree.root_id, l_state, total_width / 2);

        explorer.set_active();

        self.explorer = Some(explorer);
        self.task_list = Some(task_list);

        Ok(())
    }

    async fn update(&mut self, signal: Signal) -> color_eyre::Result<Option<Signal>> {
        let explorer = self
            .explorer
            .as_mut()
            .expect("This should have already been initialized");

        let _task_list = self
            .task_list
            .as_mut()
            .expect("This should have already been initialized");

        match signal {
            Signal::MoveDown => {
                explorer.state.select_next();
                // self.update_views_from_zettel_list_selection().await?;
            }

            Signal::MoveUp => {
                explorer.state.select_previous();
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

        let explorer = self.explorer.as_mut().unwrap();
        let task_list = self.task_list.as_mut().unwrap();

        frame.render_stateful_widget(&explorer.render_list, explorer_rect, &mut explorer.state);
        frame.render_stateful_widget(&task_list.render_list, task_list_rect, &mut task_list.state);
        Ok(())
    }
}
