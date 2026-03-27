use std::{ collections::HashMap,
    ops::{Deref, DerefMut},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use kdl::KdlNode;
use strum::IntoEnumIterator;

use crate::tui::{Signal, app::Region};

#[derive(Debug, Clone)]
pub struct KeyMap(pub HashMap<Region, HashMap<Vec<KeyEvent>, Signal>>);

impl TryFrom<&KdlNode> for KeyMap {
    type Error = color_eyre::Report;

    fn try_from(value: &KdlNode) -> std::result::Result<Self, Self::Error> {
        let mut all_binds = HashMap::new();

        for region in Region::iter() {
            let mut region_binds = HashMap::new();
            let Some(binds) = value
                .children()
                .expect("Keymap must have children.")
                .get(&region.to_string())
            else {
                continue;
            };

            // now we iter through the things children
            for child in binds.iter_children() {
                let key_combo_str = child.name().to_string();
                let key_combo_str = key_combo_str.trim();

                let signal_str = child
                    .entries()
                    .first()
                    .expect("A bind must map to an entry")
                    .to_string();
                let signal_str = signal_str.trim();

                let signal: Signal = signal_str.parse().expect("Must be a \"bindable\" Signal");
                let key_combo = parse_key_sequence(key_combo_str).unwrap();

                let _ = region_binds.insert(key_combo, signal);
            }

            let _ = all_binds.insert(region, region_binds);
        }

        Ok(Self(all_binds))
    }
}

impl Deref for KeyMap {
    type Target = HashMap<Region, HashMap<Vec<KeyEvent>, Signal>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeyMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn parse_key_sequence(raw: &str) -> color_eyre::Result<Vec<KeyEvent>, String> {
    if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
        return Err(format!("Unable to parse `{raw}`"));
    }
    let raw = if raw.contains("><") {
        raw
    } else {
        let raw = raw.strip_prefix('<').unwrap_or(raw);

        raw.strip_prefix('>').unwrap_or(raw)
    };

    raw.split("><")
        .map(|seq| {
            seq.strip_prefix('<')
                .unwrap_or_else(|| seq.strip_suffix('>').map_or(seq, |s| s))
        })
        .map(parse_key_event)
        .collect()
}

fn parse_key_event(raw: &str) -> color_eyre::Result<KeyEvent, String> {
    let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        match current {
            rest if rest.starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            }
            rest if rest.starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            }
            rest if rest.starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        }
    }

    (current, modifiers)
}

fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> color_eyre::Result<KeyEvent, String> {
    let c = match raw {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" | "minuc" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = c.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => return Err(format!("Unable to parse {raw}")),
    };
    Ok(KeyEvent::new(c, modifiers))
}

#[cfg(test)]
mod test {
    use crossterm::event::{KeyEvent, KeyModifiers};
    use kdl::KdlNode;

    use crate::tui::{KeyMap, Signal, app::Region};

    #[test]
    fn test_quit_in_home_region() {
        let keymap_str = "
            keymap {
                Home {
                    q Quit
                    <Ctrl-C> Quit
                }
            }
        ";

        let kdl: &KdlNode = &keymap_str
            .parse()
            .expect("Keymap_str should be a valid KDL document");

        let keymap: KeyMap = kdl.try_into().expect("Must be a valid keymap");

        let map = keymap
            .get(&Region::Home)
            .expect("Home region must exist in keymap");

        let signal = map
            .get(&vec![KeyEvent::new_with_kind(
                crossterm::event::KeyCode::Char('q'),
                KeyModifiers::empty(),
                crossterm::event::KeyEventKind::Press,
            )])
            .expect("Must resolve to a signal");

        assert_eq!(*signal, Signal::Quit);
    }
}
