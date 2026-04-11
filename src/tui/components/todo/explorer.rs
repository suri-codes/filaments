#![allow(dead_code)]

use dto::NanoId;
use ratatui::{text::Span, widgets::ListState};

pub struct Explorer<'text> {
    pub render_list: ratatui::widgets::List<'text>,
    pub id_list: Vec<NanoId>,
    pub state: ListState,
    pub width: u16,
}

pub struct ExplorerListItem<'text> {
    name: Span<'text>,
}

// impl From<&Task> for ExplorerListItem {
//     fn from(value: &Task) -> Self {
//         Self {
//             name: Span { style: (), content: () }
//         }
//     }
// }

// impl Explorer {
//     pub async fn

// }
