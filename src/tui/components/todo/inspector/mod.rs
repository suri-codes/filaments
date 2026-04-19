use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use dto::{GroupEntity, IntoActiveModel, NanoId, TagEntity, TaskEntity};
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
    types::{KastenHandle, TodoNode, TodoNodeKind},
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
}

impl Inspector<'_> {
    pub fn new(kh: KastenHandle, node: &TodoNode) -> Self {
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

        let render_data = match node.kind {
            TodoNodeKind::Root => RenderData::Root {
                widget: Box::new(RootView::default()),
            },
            TodoNodeKind::Group(ref group) => {
                nanoid = Some(group.id.clone());

                RenderData::Group {
                    widget: Box::new(GroupView::from(&**group)),
                }
            }
            TodoNodeKind::Task(ref task) => {
                nanoid = Some(task.id.clone());

                RenderData::Task {
                    widget: Box::new(TaskView::from(&**task)),
                }
            }
        };

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

    pub fn inspect(&mut self, node: &TodoNode) {
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
                    widget: Box::new(GroupView::from(&**group)),
                }
            }
            TodoNodeKind::Task(ref task) => {
                self.inspecting = Some(task.id.clone());
                RenderData::Task {
                    widget: Box::new(TaskView::from(&**task)),
                }
            }
        }
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

    #[allow(clippy::too_many_lines)]
    async fn update(&mut self, signal: Signal) -> color_eyre::Result<Option<Signal>> {
        match signal {
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
                        .border_style(Style::default().fg(Color::Green)),
                );

                priority.set_cursor_style(Style::default().reversed());
                priority.set_cursor_line_style(Style::default().underlined());

                self.editing = Some(Edit::Priority);
                return Ok(Some(Signal::EnterRawText));
            }

            _ => {}
        }
        Ok(None)
    }

    #[allow(clippy::too_many_lines)]
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

                    let kt = self.kh.read().await;
                    match &self.render_data {
                        RenderData::Task { .. } => {
                            let _ = TaskEntity::load()
                                .filter_by_nano_id(id.clone())
                                .one(&kt.db)
                                .await?
                                .expect("Invariant Broken: Must exist")
                                .into_active_model()
                                .set_name(new_name.as_str())
                                .save(&kt.db)
                                .await?;
                        }
                        RenderData::Group { .. } => {
                            let g = GroupEntity::load()
                                .filter_by_nano_id(id.clone())
                                .with(TagEntity)
                                .one(&kt.db)
                                .await?
                                .expect("Invariant Broken: Must exist");
                            let tag_id = g.tag.as_ref().expect("Must be loaded").nano_id.clone();

                            let _ = g
                                .into_active_model()
                                .set_name(new_name.as_str())
                                .save(&kt.db)
                                .await?;

                            TagEntity::load()
                                .filter_by_nano_id(tag_id)
                                .one(&kt.db)
                                .await?
                                .expect("Invariant Broken: Must exist")
                                .into_active_model()
                                .set_name(new_name.as_str())
                                .save(&kt.db)
                                .await?;
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

                if key.code == KeyCode::Enter {
                    priority.set_cursor_style(Style::reset());
                    priority.set_cursor_line_style(Style::reset());

                    priority.set_block(
                        priority
                            .block()
                            .cloned()
                            .expect("All of them should have blocks")
                            .border_style(Style::default().fg(Color::Reset)),
                    );

                    self.editing = None;
                    Ok(Some(Signal::ExitRawText))
                } else {
                    priority.input_without_shortcuts(key);
                    Ok(None)
                }
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
