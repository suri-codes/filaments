use std::{path::PathBuf, str::FromStr};

use color_eyre::eyre::eyre;
use strum::Display;

use serde::{Deserialize, Serialize};

use crate::{tui::Region, types::ZettelId};

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

    SwitchTo {
        region: Region,
    },

    // movement
    MoveDown,
    MoveUp,

    /// Create a  New `Zettel`
    NewZettel,

    CreatedZettel {
        zid: ZettelId,
    },

    /// User asks to open a `Zettel`
    OpenZettel,
    /// The user is done editing a `Zettel`
    ClosedZettel {
        /// the id of the `Zettel` that was closed
        zid: ZettelId,
    },

    /// this is fucking temporary
    Helix {
        path: PathBuf,
    },
}

impl FromStr for Signal {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "suspend" => Self::Suspend,
            "resume" => Self::Resume,
            "quit" => Self::Quit,
            "movedown" => Self::MoveDown,
            "moveup" => Self::MoveUp,
            "openzettel" => Self::OpenZettel,
            "newzettel" => Self::NewZettel,
            _ => {
                return Err(eyre!(format!(
                    "Attempt to construct a non-user Signal from str: {s}"
                )));
            }
        })
    }
}
