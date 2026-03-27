use std::str::FromStr;

use color_eyre::eyre::eyre;
use strum::Display;

use serde::{Deserialize, Serialize};

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
}

impl FromStr for Signal {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "suspend" => Self::Suspend,
            "resume" => Self::Resume,
            "quit" => Self::Quit,
            _ => {
                return Err(eyre!(format!(
                    "Attempt to construct a non-user Signal from str: {s}"
                )));
            }
        })
    }
}
