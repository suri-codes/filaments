use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use dto::NanoId;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};
use ratatui_textarea::CursorMove;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    tui::{
        Signal,
        components::{Component, DEFAULT_NAME},
    },
    types::{Due, Group, KastenHandle, Priority, Task, TodoNode, TodoNodeKind},
};

mod rootview;
use rootview::RootView;

mod taskview;
use taskview::TaskView;

mod groupview;
use groupview::GroupView;

pub struct Inspector<'text> {
    pub render_data: RenderData<'text>,
    margins: Layout,
    block: Block<'text>,
    kh: KastenHandle,
    signal_tx: Option<UnboundedSender<Signal>>,
    is_active: bool,
    editing: Option<Edit>,
    // this is the `NanoId` of the thing we are actually inspecting
    inspecting: Option<NanoId>,
}

enum Edit {
    Name,
    Priority,
    Due,
}

impl Inspector<'_> {
    pub async fn new(kh: KastenHandle, node: &TodoNode) -> Self {
        let margins = Layout::new(Direction::Horizontal, [Constraint::Percentage(100)])
            .horizontal_margin(3)
            .vertical_margin(2);

        let block = Block::new()
            .title("[2]")
            .title("Inspector")
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .border_style(Style::new().fg(Color::Gray))
            .border_type(BorderType::Rounded);

        let mut nanoid = None;
        let kt = kh.read().await;

        let render_data = match node.kind {
            TodoNodeKind::Root => RenderData::Root {
                widget: Box::new(RootView::default()),
            },
            TodoNodeKind::Group(ref group) => {
                nanoid = Some(group.id.clone());

                RenderData::Group {
                    widget: Box::new(GroupView::from((&**group, &kt.index))),
                }
            }
            TodoNodeKind::Task(ref task) => {
                nanoid = Some(task.id.clone());

                RenderData::Task {
                    widget: Box::new(TaskView::from((&**task, &kt.index))),
                }
            }
        };

        drop(kt);

        Self {
            render_data,
            margins,
            block,
            kh,
            is_active: false,
            editing: None,
            inspecting: nanoid,
            signal_tx: None,
        }
    }

    pub fn set_active(&mut self) {
        self.is_active = true;

        self.block = Block::new()
            .title("[2]")
            .title("Inspector")
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .border_style(Style::new().fg(Color::Green))
            .border_type(BorderType::Rounded);
    }

    pub fn set_inactive(&mut self) {
        self.is_active = false;
        self.block = Block::new()
            .title("[2]")
            .title("Inspector")
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .border_style(Style::new().fg(Color::Gray))
            .border_type(BorderType::Rounded);
    }

    pub async fn inspect(&mut self, node: &TodoNode) {
        let kt = self.kh.read().await;

        self.render_data = match node.kind {
            TodoNodeKind::Root => {
                self.inspecting = None;
                RenderData::Root {
                    widget: Box::new(RootView::default()),
                }
            }
            TodoNodeKind::Group(ref group) => {
                self.inspecting = Some(group.id.clone());
                RenderData::Group {
                    widget: Box::new(GroupView::from((&**group, &kt.index))),
                }
            }
            TodoNodeKind::Task(ref task) => {
                self.inspecting = Some(task.id.clone());
                RenderData::Task {
                    widget: Box::new(TaskView::from((&**task, &kt.index))),
                }
            }
        }
    }

    async fn refresh(&mut self) {
        // cheaper to clone this than the node
        let kh = self.kh.clone();
        let kt = kh.read().await;

        let Some(ref inspecting) = self.inspecting else {
            return;
        };
        let node = kt.todo_tree.get_node_by_nano_id(inspecting).data();
        self.inspect(node).await;

        drop(kt);
    }
}

pub enum RenderData<'text> {
    Root { widget: Box<RootView<'text>> },
    Task { widget: Box<TaskView<'text>> },
    Group { widget: Box<GroupView<'text>> },
}

#[async_trait]
impl Component for Inspector<'_> {
    fn register_signal_handler(&mut self, tx: UnboundedSender<Signal>) -> color_eyre::Result<()> {
        self.signal_tx = Some(tx);
        Ok(())
    }

    async fn update(&mut self, signal: Signal) -> color_eyre::Result<Option<Signal>> {
        match signal {
            Signal::SwitchTo {
                page: crate::tui::Page::Zk,
            } => {
                self.is_active = false;

                self.set_inactive();
            }

            Signal::EditName => {
                let name = match &mut self.render_data {
                    RenderData::Root { widget: _ } => return Ok(None),
                    RenderData::Task { widget } => &mut widget.name,
                    RenderData::Group { widget } => &mut widget.name,
                };

                name.set_block(
                    name.block()
                        .cloned()
                        .expect("All of them should have blocks")
                        .border_style(Style::default().fg(Color::Green)),
                );

                if name.lines()[0].as_str().contains(DEFAULT_NAME) {
                    name.delete_line_by_end();
                } else {
                    name.move_cursor(CursorMove::End);
                }

                name.set_cursor_style(Style::default().reversed());
                name.set_cursor_line_style(Style::default().underlined());

                self.editing = Some(Edit::Name);
                return Ok(Some(Signal::EnterRawText));
            }
            Signal::EditPriority => {
                let priority = match &mut self.render_data {
                    RenderData::Root { widget: _ } => return Ok(None),
                    RenderData::Task { widget } => &mut widget.priority,
                    RenderData::Group { widget } => &mut widget.priority,
                };

                priority.set_block(
                    priority
                        .block()
                        .cloned()
                        .expect("All of them should have blocks")
                        .border_style(Style::default().fg(Color::Yellow)),
                );

                priority.set_cursor_style(Style::default().reversed());
                priority.set_cursor_line_style(Style::default().underlined());
                priority.move_cursor(CursorMove::WordBack);
                priority.delete_line_by_end();

                self.editing = Some(Edit::Priority);
                return Ok(Some(Signal::EnterRawText));
            }

            Signal::EditDue => {
                let kt = self.kh.read().await;

                let Some(ref inspecting) = self.inspecting else {
                    return Ok(None);
                };

                // if its finished, we arent going to edit the due date lol
                if let TodoNodeKind::Task(task) =
                    &kt.todo_tree.get_node_by_nano_id(inspecting).data().kind
                    && task.finished_at.is_some()
                {
                    return Ok(None);
                }

                drop(kt);

                let due = match &mut self.render_data {
                    RenderData::Task { widget } => &mut widget.due_finished_at,
                    _ => return Ok(None),
                };

                due.set_block(
                    due.block()
                        .cloned()
                        .expect("All of them should have blocks")
                        .border_style(Style::default().fg(Color::Green)),
                );

                due.set_cursor_style(Style::default().reversed());
                due.set_cursor_line_style(Style::default().underlined());
                due.move_cursor(CursorMove::WordBack);
                due.delete_line_by_end();

                self.editing = Some(Edit::Due);
                return Ok(Some(Signal::EnterRawText));
            }

            Signal::Refresh => {
                self.refresh().await;
            }

            Signal::OpenZettel if self.is_active => {
                let Some(ref curr) = self.inspecting else {
                    return Ok(None);
                };

                let kt = self.kh.read().await;

                let node = kt.todo_tree.get_node_by_nano_id(curr).data();

                let zid = match &node.kind {
                    TodoNodeKind::Root => return Ok(None),
                    TodoNodeKind::Group(group) => &group.zettel.id,
                    TodoNodeKind::Task(task) => &task.zettel.id,
                };

                let path = kt.index.get_zod(zid).path.clone();
                drop(kt);
                return Ok(Some(Signal::Helix { path }));
            }

            _ => {}
        }
        Ok(None)
    }

    #[expect(clippy::too_many_lines)]
    async fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Signal>> {
        let signal_tx = self
            .signal_tx
            .as_mut()
            .expect("Invariant Broken, signal_tx must be initialized");

        match self.editing {
            Some(Edit::Name) => {
                let name = match &mut self.render_data {
                    RenderData::Root { widget: _ } => return Ok(None),
                    RenderData::Task { widget } => &mut widget.name,
                    RenderData::Group { widget } => &mut widget.name,
                };

                if key.code == KeyCode::Enter {
                    name.set_cursor_style(Style::reset());
                    name.set_cursor_line_style(Style::reset());
                    name.set_block(
                        name.block()
                            .cloned()
                            .expect("All of them should have blocks")
                            .border_style(Style::default().fg(Color::Reset)),
                    );
                    self.editing = None;
                    signal_tx.send(Signal::ExitRawText)?;

                    let new_name = name.lines()[0].clone();
                    let id = self
                        .inspecting
                        .clone()
                        .expect("Invariant Broken, this must be some id");

                    let mut kt = self.kh.write().await;
                    match &self.render_data {
                        RenderData::Task { .. } => {
                            Task::alter_name(id.clone(), new_name, &mut kt).await?;
                        }
                        RenderData::Group { .. } => {
                            Group::alter_name(id.clone(), new_name, &mut kt).await?;
                        }
                        RenderData::Root { .. } => unreachable!("Already returned above"),
                    }

                    drop(kt);

                    Ok(Some(Signal::Refresh))
                } else {
                    name.input_without_shortcuts(key);
                    Ok(None)
                }
            }
            Some(Edit::Priority) => {
                let priority = match &mut self.render_data {
                    RenderData::Root { widget: _ } => return Ok(None),
                    RenderData::Task { widget } => &mut widget.priority,
                    RenderData::Group { widget } => &mut widget.priority,
                };

                // we dont want them entering into this
                if key.code != KeyCode::Enter {
                    priority.input_without_shortcuts(key);
                }

                let priority_str = priority.lines()[0].as_str();

                if let Ok(prio) = Priority::try_from(priority_str) {
                    priority.set_block(
                        priority
                            .block()
                            .cloned()
                            .expect("All of them should have blocks")
                            .border_style(Style::default().fg(Color::Green)),
                    );

                    if key.code == KeyCode::Enter {
                        self.editing = None;
                        signal_tx.send(Signal::ExitRawText)?;

                        priority.set_cursor_style(Style::reset());
                        priority.set_cursor_line_style(Style::reset());

                        priority.set_block(
                            priority
                                .block()
                                .cloned()
                                .expect("All of them should have blocks")
                                .border_style(Style::default().fg(Color::Reset)),
                        );

                        let id = self
                            .inspecting
                            .clone()
                            .expect("Invariant Broken, this must be some id");

                        let kt = self.kh.read().await;

                        match &self.render_data {
                            RenderData::Task { .. } => {
                                Task::alter_priority(id.clone(), prio, &kt).await?;
                            }
                            RenderData::Group { .. } => {
                                Group::alter_priority(id.clone(), prio, &kt).await?;
                            }
                            RenderData::Root { .. } => unreachable!("Already returned above"),
                        }

                        drop(kt);

                        return Ok(Some(Signal::Refresh));
                    }
                } else {
                    priority.set_block(
                        priority
                            .block()
                            .cloned()
                            .expect("All of them should have blocks")
                            .border_style(Style::default().fg(Color::Red)),
                    );
                }

                Ok(None)
            }

            Some(Edit::Due) => {
                let due = match &mut self.render_data {
                    RenderData::Task { widget } => &mut widget.due_finished_at,
                    _ => return Ok(None),
                };

                if key.code != KeyCode::Enter {
                    due.input_without_shortcuts(key);
                }

                let due_str = due.lines()[0].as_str();

                if let Ok(new_due) = Due::try_from(due_str) {
                    due.set_block(
                        due.block()
                            .cloned()
                            .expect("All of them should have blocks")
                            .border_style(Style::default().fg(Color::Green)),
                    );

                    if key.code == KeyCode::Enter {
                        self.editing = None;
                        signal_tx.send(Signal::ExitRawText)?;

                        due.set_cursor_style(Style::reset());
                        due.set_cursor_line_style(Style::reset());

                        due.set_block(
                            due.block()
                                .cloned()
                                .expect("All of them should have blocks")
                                .border_style(Style::default().fg(Color::Reset)),
                        );

                        let id = self
                            .inspecting
                            .clone()
                            .expect("Invariant Broken, this must be some id");

                        let kt = self.kh.read().await;

                        match &self.render_data {
                            RenderData::Task { .. } => {
                                Task::alter_due(id.clone(), new_due.into(), &kt).await?;
                            }
                            _ => unreachable!("Already returned above"),
                        }

                        drop(kt);

                        return Ok(Some(Signal::Refresh));
                    }
                } else {
                    due.set_block(
                        due.block()
                            .cloned()
                            .expect("All of them should have blocks")
                            .border_style(Style::default().fg(Color::Red)),
                    );
                }

                Ok(None)
            }

            None => return Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        frame.render_widget(self.block.clone(), area);

        let area = self.margins.split(area)[0];

        match &self.render_data {
            RenderData::Root { widget } => frame.render_widget(*widget.clone(), area),
            RenderData::Task { widget } => frame.render_widget(*widget.clone(), area),
            RenderData::Group { widget } => frame.render_widget(*widget.clone(), area),
        }

        Ok(())
    }
}
