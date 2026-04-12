use async_trait::async_trait;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Size},
    style::{Color, Stylize},
    widgets::{Block, ListState},
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
    explorer_right: Layout,
    inspector_task_list: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            explorer_right: Layout::horizontal(vec![
                Constraint::Percentage(40),
                Constraint::Fill(100),
            ]),
            inspector_task_list: Layout::vertical(vec![
                Constraint::Percentage(30),
                Constraint::Fill(100),
            ]),
        }
    }
}

struct LayoutSplit {
    explorer: Rect,
    inspector: Rect,
    task_list: Rect,
}

impl Layouts {
    fn split(&self, area: Rect) -> LayoutSplit {
        let rects = self.explorer_right.split(area);
        let r_rects = self.inspector_task_list.split(rects[1]);

        LayoutSplit {
            explorer: rects[0],
            inspector: r_rects[0],
            task_list: r_rects[1],
        }
    }
}

#[async_trait]
impl Component for Todo<'_> {
    async fn init(&mut self, area: Size) -> color_eyre::Result<()> {
        let tree = &self.kh.read().await.todo_tree;
        let splits = self.layouts.split(Rect::new(0, 0, area.width, area.height));

        let mut l_state = ListState::default();

        l_state.select_first();

        let mut explorer = Explorer::new(tree, &tree.root_id, l_state, splits.explorer.width);
        let mut task_list = TaskList::new(tree, &tree.root_id, l_state, splits.task_list.width);

        // explorer.set_active();
        explorer.set_inactive();
        // task_list.set_inactive();
        task_list.set_active();

        self.explorer = Some(explorer);
        self.task_list = Some(task_list);

        Ok(())
    }

    async fn update(&mut self, signal: Signal) -> color_eyre::Result<Option<Signal>> {
        let _explorer = self
            .explorer
            .as_mut()
            .expect("This should have already been initialized");

        let task_list = self
            .task_list
            .as_mut()
            .expect("This should have already been initialized");

        match signal {
            Signal::MoveDown => {
                // explorer.state.select_next();
                task_list.state.select_next();
                // self.update_views_from_zettel_list_selection().await?;
            }

            Signal::MoveUp => {
                // explorer.state.select_previous();
                task_list.state.select_previous();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let explorer = self.explorer.as_mut().unwrap();
        let task_list = self.task_list.as_mut().unwrap();

        let splits = self.layouts.split(area);

        frame.render_stateful_widget(&explorer.render_list, splits.explorer, &mut explorer.state);
        frame.render_stateful_widget(
            &task_list.render_list,
            splits.task_list,
            &mut task_list.state,
        );
        frame.render_widget(Block::new().bg(Color::Green), splits.inspector);
        Ok(())
    }
}
