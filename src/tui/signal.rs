use std::path::PathBuf;

use strum::Display;

use serde::{Deserialize, Serialize};

use crate::{
    tui::Page,
    types::{Link, ZettelId},
};

/// The varying signals that can be emitted.
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Signal {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,

    /// Request a refresh of the components being displayed due
    /// to an update to the `Kasten`
    Refresh,

    SwitchTo {
        page: Page,
    },

    // movement
    MoveDown,
    MoveUp,

    /// Create a  New `Zettel`
    NewZettel,

    /// This zettel was created (filaments specific)
    CreatedZettel {
        zid: ZettelId,
    },

    /// Set the links for this `ZettelId` (filaments specific)
    SetLinks {
        zid: ZettelId,
        links: Vec<Link>,
    },

    /// User asks to open a `Zettel`
    OpenZettel,
    /// The user is done editing a `Zettel`
    ClosedZettel {
        /// the id of the `Zettel` that was closed
        zid: ZettelId,
    },

    /// Create a new `Group` inside the currently selected group
    NewSubGroup,

    /// Create a new `Group` in the current scope
    NewGroup,

    /// Create a new `Task`
    NewTask,

    /// Edit the name of a `Task` or a `Group`.
    /// Only works with the inspector
    EditName,

    /// this is fucking temporary
    Helix {
        path: PathBuf,
    },
}

// impl FromStr for Signal {
//     type Err = color_eyre::Report;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         Ok(match s.to_lowercase().as_str() {
//             "suspend" => Self::Suspend,
//             "resume" => Self::Resume,
//             "quit" => Self::Quit,
//             "movedown" => Self::MoveDown,
//             "moveup" => Self::MoveUp,
//             "openzettel" => Self::OpenZettel,
//             "newzettel" => Self::NewZettel,
//             "newgroup"
//             _ => {
//                 return Err(eyre!(format!(
//                     "Attempt to construct a non-user Signal from str: {s}"
//                 )));
//             }
//         })
//     }
// }
