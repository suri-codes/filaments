use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Widget},
};

use crate::types::{TodoNode, TodoNodeKind};

mod rootview;
use rootview::RootView;

mod taskview;
use taskview::TaskView;

mod groupview;
use groupview::GroupView;

pub struct Inspector<'text> {
    render_data: RenderData<'text>,
    margins: Layout,
    block: Block<'text>,
}

impl Inspector<'_> {
    pub fn set_active(&mut self) {
        self.block = Block::new()
            .title("[2]")
            .title("Inspector")
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .border_style(Style::new().fg(Color::Green))
            .border_type(BorderType::Rounded);
    }

    pub fn set_inactive(&mut self) {
        self.block = Block::new()
            .title("[2]")
            .title("Inspector")
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .border_style(Style::new().fg(Color::Gray))
            .border_type(BorderType::Rounded);
    }
}

enum RenderData<'text> {
    Root { widget: Box<RootView<'text>> },
    Task { widget: Box<TaskView<'text>> },
    Group { widget: Box<GroupView<'text>> },
}

impl From<&TodoNode> for Inspector<'_> {
    fn from(value: &TodoNode) -> Self {
        let margins = Layout::new(Direction::Horizontal, [Constraint::Percentage(100)])
            .horizontal_margin(3)
            .vertical_margin(2);

        let block = Block::new()
            .title("[2]")
            .title("Inspector")
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .border_style(Style::new().fg(Color::Gray))
            .border_type(BorderType::Rounded);
        match value.kind {
            TodoNodeKind::Root => Self {
                render_data: RenderData::Root {
                    widget: Box::new(RootView::default()),
                },
                margins,
                block,
            },
            TodoNodeKind::Group(ref group) => Self {
                render_data: RenderData::Group {
                    widget: Box::new(GroupView::from(&**group)),
                },
                margins,
                block,
            },
            TodoNodeKind::Task(ref task) => Self {
                render_data: RenderData::Task {
                    widget: Box::new(TaskView::from(&**task)),
                },
                margins,
                block,
            },
        }
    }
}

impl Widget for &Inspector<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.block.clone().render(area, buf);

        let area = self.margins.split(area)[0];

        match &self.render_data {
            RenderData::Root { widget } => widget.clone().render(area, buf),
            RenderData::Task { widget } => widget.clone().render(area, buf),
            RenderData::Group { widget } => widget.clone().render(area, buf),
        }
    }
}
