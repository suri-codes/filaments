use async_trait::async_trait;
use dto::NanoId;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Size},
    widgets::ListState,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, info};

use crate::{
    tui::{Page, Signal, components::Component},
    types::{Group, KastenHandle},
};

mod explorer;
use explorer::Explorer;
mod tasklist;
use tasklist::TaskList;

mod inspector;
use inspector::Inspector;

pub struct Todo<'text> {
    #[expect(dead_code)]
    signal_tx: Option<UnboundedSender<Signal>>,
    kh: KastenHandle,
    layouts: Layouts,
    explorer: Option<Explorer<'text>>,
    task_list: Option<TaskList<'text>>,
    inspector: Option<Inspector<'text>>,

    area: Size,
    active: TodoRegion,
}

/// The different regions inside the `Todo` component
#[derive(
    Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display,
)]
pub enum TodoRegion {
    Inspector,
    TaskList,
    #[default]
    Explorer,
}

impl Todo<'_> {
    pub fn new(kh: KastenHandle) -> Self {
        Self {
            kh,
            layouts: Layouts::default(),
            signal_tx: None,
            explorer: None,
            task_list: None,
            inspector: None,
            area: Size::default(),
            active: TodoRegion::default(),
        }
    }

    pub async fn refresh(&mut self) {
        let explorer = self
            .explorer
            .as_mut()
            .expect("This should have already been init.ialized");
        let task_list = self
            .task_list
            .as_mut()
            .expect("This should have already been initialized");

        let explorer_selection = explorer
            .state
            .selected()
            .and_then(|idx| explorer.id_list.get(idx));
        let task_list_selection = task_list
            .state
            .selected()
            .and_then(|idx| task_list.id_list.get(idx));
        let kt = self.kh.read().await;
        let tree = &kt.todo_tree;

        let splits = self
            .layouts
            .split(Rect::new(0, 0, self.area.width, self.area.height));

        let l_state = ListState::default();

        //TODO: instead of tree.root_id this probably should be scope.
        let mut explorer = Explorer::new(tree, &tree.root_id, l_state, splits.explorer.width);
        let mut task_list = TaskList::new(tree, &tree.root_id, l_state, splits.task_list.width);

        drop(kt);

        let explorer_selection_idx =
            explorer_selection.and_then(|id| explorer.id_list.iter().position(|e| id == e));

        let task_list_selection_idx =
            task_list_selection.and_then(|id| task_list.id_list.iter().position(|e| id == e));

        explorer.state.select(explorer_selection_idx);
        task_list.state.select(task_list_selection_idx);

        match self.active {
            TodoRegion::Inspector => {
                explorer.set_inactive();
                task_list.set_inactive();
            }
            TodoRegion::TaskList => {
                explorer.set_inactive();
                task_list.set_active();
            }
            TodoRegion::Explorer => {
                explorer.set_active();
                task_list.set_inactive();
            }
        }

        self.explorer = Some(explorer);
        self.task_list = Some(task_list);
        self.update_inspector_from_selection().await;
    }

    async fn update_inspector_from_selection(&mut self) {
        let explorer = self
            .explorer
            .as_mut()
            .expect("This should have already been init.ialized");
        let task_list = self
            .task_list
            .as_mut()
            .expect("This should have already been initialized");
        let inspector = self
            .inspector
            .as_mut()
            .expect("This should have already been initialized");

        let selected_node_id = match self.active {
            TodoRegion::TaskList => {
                let Some(idx) = task_list.state.selected() else {
                    return;
                };
                task_list.id_list.get(idx)
            }
            TodoRegion::Explorer => {
                let Some(idx) = explorer.state.selected() else {
                    return;
                };
                explorer.id_list.get(idx)
            }
            TodoRegion::Inspector => return,
        };

        let Some(selected_node_id) = selected_node_id else {
            return;
        };
        let tree = &self.kh.read().await.todo_tree.tree;

        *inspector = tree
            .get(selected_node_id)
            .expect("Nodeid must be valid")
            .data()
            .into();
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
        self.area = area;
        let tree = &self.kh.read().await.todo_tree;
        let splits = self.layouts.split(Rect::new(0, 0, area.width, area.height));

        let mut l_state = ListState::default();

        l_state.select_first();

        let mut explorer = Explorer::new(tree, &tree.root_id, l_state, splits.explorer.width);
        let mut task_list = TaskList::new(tree, &tree.root_id, l_state, splits.task_list.width);

        let first = tree
            .tree
            .get(
                task_list
                    .id_list
                    .first()
                    .unwrap_or_else(|| tree.tree.root_node_id().expect("Root node must exist")),
            )
            .expect("Node id must be valid");

        let mut inspector: Inspector<'_> = first.data().into();

        explorer.set_inactive();
        inspector.set_inactive();
        task_list.set_inactive();
        self.explorer = Some(explorer);
        self.task_list = Some(task_list);
        self.inspector = Some(inspector);

        // match self.active {

        //     ins

        // }

        Ok(())
    }

    async fn update(&mut self, signal: Signal) -> color_eyre::Result<Option<Signal>> {
        let explorer = self
            .explorer
            .as_mut()
            .expect("This should have already been init.ialized");

        let task_list = self
            .task_list
            .as_mut()
            .expect("This should have already been initialized");
        let inspector = self
            .inspector
            .as_mut()
            .expect("This should have already been initialized");

        match signal {
            Signal::SwitchTo {
                page: Page::Todo(region),
            } => {
                self.active = region;
                match region {
                    TodoRegion::Inspector => {
                        inspector.set_active();
                        explorer.set_inactive();
                        task_list.set_inactive();
                    }
                    TodoRegion::TaskList => {
                        inspector.set_inactive();
                        explorer.set_inactive();
                        task_list.set_active();
                    }
                    TodoRegion::Explorer => {
                        explorer.set_active();
                        task_list.set_inactive();
                        inspector.set_inactive();
                    }
                }

                self.update_inspector_from_selection().await;
            }
            Signal::MoveDown => {
                match self.active {
                    TodoRegion::TaskList => {
                        task_list.state.select_next();
                    }
                    TodoRegion::Explorer => {
                        explorer.state.select_next();
                    }
                    TodoRegion::Inspector => {
                        return Ok(None);
                    }
                }

                self.update_inspector_from_selection().await;
            }

            Signal::MoveUp => {
                match self.active {
                    TodoRegion::TaskList => {
                        task_list.state.select_previous();
                    }
                    TodoRegion::Explorer => {
                        explorer.state.select_previous();
                    }
                    TodoRegion::Inspector => return Ok(None),
                }

                self.update_inspector_from_selection().await;
            }

            Signal::NewGroup => {
                if self.active != TodoRegion::Explorer {
                    return Ok(None);
                }
                debug!("Creating Group!");
                let mut kt = self.kh.write().await;
                let group = Group::new(NanoId::default().to_string(), None, &mut kt).await?;
                drop(kt);
                debug!("Created group: {group:#?}");
                return Ok(Some(Signal::Refresh));
            }

            Signal::Refresh => {
                self.refresh().await;
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
        frame.render_widget(self.inspector.as_ref().unwrap(), splits.inspector);
        Ok(())
    }
}
