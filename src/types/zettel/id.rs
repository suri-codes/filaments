use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Error, eyre};
use dto::NanoId;
use serde::{Deserialize, Serialize};

/// A `ZettelId` is essentially a `NanoId`,
/// with some `Zettel` specific helpers written
/// onto it
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZettelId(pub(super) NanoId);

impl From<&str> for ZettelId {
    fn from(value: &str) -> Self {
        Self(NanoId::from(value))
    }
}

impl From<&NanoId> for ZettelId {
    fn from(value: &NanoId) -> Self {
        value.clone().into()
    }
}

impl From<NanoId> for ZettelId {
    fn from(value: NanoId) -> Self {
        Self(value)
    }
}

impl TryFrom<PathBuf> for ZettelId {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let path = value.as_path();
        path.try_into()
    }
}

impl TryFrom<&Path> for ZettelId {
    type Error = Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let extension = value
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| eyre!("Unable to turn file extension into string".to_owned(),))?;

        if extension != "md" {
            return Err(eyre!(format!("Wrong extension: {extension}, expected .md")));
        }

        let id: Self = (value
            .file_name()
            .ok_or_else(|| eyre!("Invalid File Name!".to_owned()))?
            .to_str()
            .ok_or_else(|| eyre!("File Name cannot be translated into str!".to_owned(),))?
            .strip_suffix(".md")
            .expect("we statically verify this right above")
            .split('-'))
        .next()
        .ok_or_else(|| eyre!("Unable to get the first part of the file name!"))?
        .into();

        Ok(id)
    }
}

impl Display for ZettelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

impl From<ZettelId> for NanoId {
    fn from(value: ZettelId) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[tokio::test]
    async fn test_zettel_id_parsing_from_path() {
        let path = PathBuf::from("/what/the/fuck/are/you/abcdef-doing-monkey.md");

        let id: ZettelId = path
            .try_into()
            .expect("Should be able to parse the test path just file");

        assert_eq!(id.0, "abcdef".into());

        let path = PathBuf::from("/what/the/fuck/are/you/abcdef.md");

        let id: ZettelId = path
            .try_into()
            .expect("Should be able to parse the test path just file");

        assert_eq!(id.0, "abcdef".into());
    }
}
