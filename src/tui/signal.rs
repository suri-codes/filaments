use std::path::PathBuf;

use dto::NanoId;
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

    /// Edit the `Priority` of a `Task` or a `Group`.
    /// Only works with the inspector
    EditPriority,

    /// Edit the `DueDate` of a `Task`
    /// Only works with the inspector
    EditDue,

    /// Toggle whether a `Task` is finished or not.
    ToggleFinish,

    /// Internal Signal that tells the app to resume interpreting keys
    ExitRawText,

    /// Internal Signal that tells the app to stop interpreting keys
    /// as signals
    EnterRawText,

    /// Randomly change the color of a `Group`!
    RandomColor,

    /// this is fucking temporary
    Helix {
        path: PathBuf,
    },

    /// Requests the `Explorer` to select the following `NanoId`.
    Select {
        nanoid: NanoId,
    },
}
