//! Filaments
//! My (suri.codes) personal-knowledge-system, with deeply integrated task tracking and long term goal planning capabilities.
//!

mod errors;
mod tui;

fn main() -> color_eyre::Result<()> {
    errors::init()?;

    Ok(())
}
