use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};
use strum::IntoEnumIterator;

use crate::tui::app::Region;

#[derive(Debug, Clone)]
pub struct Switcher<'text> {
    line: Line<'text>,
    layouts: Layouts,
}

impl Switcher<'_> {
    pub fn select_region(&mut self, region: Region) {
        self.line = Region::iter()
            .map(|r| {
                Span::from(format!(" {r} ")).style({
                    if r == region {
                        Style::new().black().on_blue()
                    } else {
                        Style::new().black().on_gray()
                    }
                })
            })
            .collect::<Line>();
    }
}

#[derive(Debug, Clone)]
struct Layouts {
    overlay: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            overlay: Layout::vertical(vec![Constraint::Fill(99), Constraint::Min(1)]),
        }
    }
}

impl Default for Switcher<'_> {
    fn default() -> Self {
        let line = Region::iter()
            .map(|r| Span::from(format!(" {r} ")).style(Style::default().bg(Color::DarkGray)))
            .collect::<Line>();

        Self {
            line,
            layouts: Layouts::default(),
        }
    }
}

impl Widget for Switcher<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rect = self.layouts.overlay.split(area)[1];

        self.line.render(rect, buf);
    }
}
