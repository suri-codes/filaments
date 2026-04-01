use ratatui::{
    layout::{Constraint, Layout},
    text::Text,
    widgets::Widget,
};

use crate::types::Zettel;

/// A `Widget` that represents a `Zettel`
#[derive(Debug, Clone)]
pub struct ZettelView<'text> {
    title: Text<'text>,
    tags: Text<'text>,
    created_at: Text<'text>,
    modified_at: Text<'text>,
    id: Text<'text>,
    layouts: Layouts,
}

impl From<&Zettel> for ZettelView<'_> {
    fn from(value: &Zettel) -> Self {
        Self {
            title: Text::from(value.title.clone()),
            tags: Text::from(
                value
                    .tags
                    .iter()
                    .fold("  ".to_owned(), |acc, t| format!("{acc} {}", t.name)),
            ),
            created_at: Text::from(value.created_at()),
            modified_at: Text::from(value.modified_at()),
            id: Text::from(value.id.to_string()),
            layouts: Layouts::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct Layouts {
    left_right: Layout,
    title_tags: Layout,
    cr_mod_id: Layout,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            left_right: Layout::horizontal(vec![
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ]),

            title_tags: Layout::vertical(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]),
            cr_mod_id: Layout::vertical(vec![
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]),
        }
    }
}

impl Widget for ZettelView<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (title_rect, tags_rect, created_rect, modified_rect, id_rect) = {
            let rects = self.layouts.left_right.split(area);

            let (left, right) = (rects[0], rects[1]);

            let l_rects = self.layouts.title_tags.split(left);

            let r_rects = self.layouts.cr_mod_id.split(right);

            (l_rects[0], l_rects[1], r_rects[0], r_rects[1], r_rects[2])
        };

        self.title.render(title_rect, buf);
        self.tags.render(tags_rect, buf);
        self.created_at.render(created_rect, buf);
        self.modified_at.render(modified_rect, buf);
        self.id.render(id_rect, buf);
    }
}
