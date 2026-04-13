use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::tui::Signal;

#[derive(Debug, Deserialize, Serialize)]
pub struct RonConfig {
    pub directory: PathBuf,
    pub global_key_binds: HashMap<String, Signal>,
    pub zk: ZkConfig,
    pub todo: TodoConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ZkConfig {
    pub keybinds: HashMap<String, Signal>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TodoConfig {
    pub explorer: ExplorerConfig,
    pub inspector: InspectorConfig,
    pub tasklist: TaskListConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExplorerConfig {
    pub keybinds: HashMap<String, Signal>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct InspectorConfig {
    pub keybinds: HashMap<String, Signal>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TaskListConfig {
    pub keybinds: HashMap<String, Signal>,
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{collections::HashMap, path::PathBuf};

    #[test]
    fn fucking_around_with_ron() {
        let x = RonConfig {
            directory: PathBuf::from("some/notes/dir"),
            global_key_binds: HashMap::from([("<Ctrl-C>".to_string(), Signal::Quit)]),
            zk: ZkConfig {
                keybinds: HashMap::from([
                    ("<Enter>".to_string(), Signal::OpenZettel),
                    ("<Esc>".to_string(), Signal::MoveDown),
                ]),
            },
            todo: TodoConfig {
                explorer: ExplorerConfig {
                    keybinds: HashMap::from([
                        ("<Space>".to_string(), Signal::NewZettel),
                        ("<Esc>".to_string(), Signal::MoveUp),
                    ]),
                },
                inspector: InspectorConfig {
                    keybinds: HashMap::from([
                        ("<Space>".to_string(), Signal::NewZettel),
                        ("<Esc>".to_string(), Signal::MoveUp),
                    ]),
                },
                tasklist: TaskListConfig {
                    keybinds: HashMap::from([
                        ("<Space>".to_string(), Signal::NewZettel),
                        ("<Esc>".to_string(), Signal::MoveUp),
                    ]),
                },
            },
        };

        let ron_string = ron::ser::to_string_pretty(&x, ron::ser::PrettyConfig::default())
            .expect("failed to serialize");

        println!("{ron_string}");
    }
}
