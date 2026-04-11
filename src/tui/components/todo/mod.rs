use async_trait::async_trait;
use color_eyre::eyre::Result;
use dto::{
    ColumnTrait as _, GroupColumns, GroupEntity, QueryFilter as _, TagEntity, TaskEntity,
    ZettelEntity,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Size},
    style::{Color, Stylize},
    widgets::Block,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    tui::{Signal, components::Component},
    types::KastenHandle,
};

mod explorer;

#[expect(dead_code)]
pub struct Todo {
    signal_tx: Option<UnboundedSender<Signal>>,
    kh: KastenHandle,
    layouts: Layouts,
}

impl Todo {
    pub async fn new(kh: KastenHandle) -> Result<Self> {
        let kt = kh.read().await;

        let _roots = GroupEntity::load()
            .with(TagEntity)
            .with(TaskEntity)
            .with((ZettelEntity, TagEntity))
            .filter(GroupColumns::ParentGroupId.is_null())
            .all(&kt.db)
            .await?;

        drop(kt);

        Ok(Self {
            kh,
            layouts: Layouts::default(),
            signal_tx: None,
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
impl Component for Todo {
    async fn init(&mut self, area: Size) -> color_eyre::Result<()> {
        let _ = area; // to appease clippy

        Ok(())
    }

    async fn update(&mut self, _signal: Signal) -> color_eyre::Result<Option<Signal>> {
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        frame.render_widget(Block::new().bg(Color::Red), area);

        Ok(())
    }
}
