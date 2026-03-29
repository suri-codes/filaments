use std::{fmt::Display, path::Path};

// use chrono::format::StrftimeItems;
use color_eyre::eyre::{Result, eyre};
use egui_graphs::Node;
use serde::{Deserialize, Serialize};
use tokio::fs;

use dto::DateTime;

use crate::types::{Link, Zettel};

const DATE_FMT_STR: &str = "%Y-%m-%d %I:%M:%S %p";

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub created_at: DateTime,
    pub tag_strings: Vec<String>,
}

impl FrontMatter {
    pub fn new(
        title: impl Into<String>,
        created_at: DateTime,
        tag_strings: Vec<impl Into<String>>,
    ) -> Self {
        let tag_strings = tag_strings.into_iter().map(Into::into).collect();

        Self {
            title: title.into(),
            created_at,
            tag_strings,
        }
    }

    /// Apply the features of `FrontMatter` onto a
    /// `Node`
    pub fn apply_node_transform(&self, node: &mut Node<Zettel, Link>) {
        node.set_label(self.title.clone());
        let disp = node.display_mut();
        disp.radius = 100.0;
    }

    /// Reads in file and returns the front matter as well as the content after it.
    /// expected format for front matter as follows
    ///```md
    /// ---
    /// Title: LOL
    /// Date: 2025-01-01 12:50:19 AM
    /// Tags: Daily barber
    /// ---
    /// ```
    pub async fn extract_from_file(path: impl AsRef<Path>) -> Result<(Self, String)> {
        let path = path.as_ref();
        let string = fs::read_to_string(path).await?;
        Self::extract_from_str(&string)
    }

    /// Returns the front matter as well as the content after it.
    /// expected format for front matter as follows
    ///```md
    /// ---
    /// Title: LOL
    /// Date: 2025-01-01 12:50:19 AM
    /// Tags: Daily barber
    /// ---
    /// ```
    pub fn extract_from_str(string: impl Into<String>) -> Result<(Self, String)> {
        let string: String = string.into();
        // we just want to strictly match this, else we error

        let lines: Vec<_> = string.lines().collect();

        let delim_check = |line_number: usize| -> Result<()> {
            let delim = lines
                .get(line_number)
                .ok_or_else(|| eyre!(format!("Line Number {line_number} doesnt exist!")))?
                .trim();
            if delim != "---" {
                return Err(eyre!("FrontMatter Deliminator Corrupted!"));
            }
            Ok(())
        };

        // check first line
        delim_check(0)?;

        //extract name
        let title = lines
            .get(1)
            .ok_or_else(|| eyre!("Title line doesn't exist!".to_owned()))?
            .strip_prefix("Title: ")
            .ok_or_else(|| eyre!("Title line doesn't start with \"Title: \" ".to_owned(),))?;

        let created_at = lines
            .get(2)
            .ok_or_else(|| eyre!("Date line doesn't exist!".to_owned()))?
            .strip_prefix("Date: ")
            .ok_or_else(|| eyre!("Date line doesn't start with \"Date: \" ".to_owned(),))
            .map(|date_str| DateTime::parse_from_str(date_str, DATE_FMT_STR))?
            .map_err(|err| eyre!(err.to_string()))?;

        let tag_strings: Vec<String> = lines
            .get(3)
            .ok_or_else(|| eyre!("Tag line doesn't exist!".to_owned()))?
            .strip_prefix("Tags: ")
            .ok_or_else(|| eyre!("Tag line doesn't start with \"Tags: \" ".to_owned(),))?
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        delim_check(4)?;

        let remaining = lines[5..].join("\n");

        Ok((Self::new(title, created_at, tag_strings), remaining))
    }
}

impl Display for FrontMatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "---")?;
        writeln!(f, "Title: {}", self.title)?;
        writeln!(f, "Date: {}", self.created_at.format(DATE_FMT_STR))?;
        write!(f, "Tags: ")?;

        for tag in &self.tag_strings {
            write!(f, "{tag} ")?;
        }

        writeln!(f, "\n---")
    }
}

#[cfg(test)]
mod tests {

    use dto::DateTime;

    use crate::types::{FrontMatter, frontmatter::DATE_FMT_STR};

    // use crate::{FrontMatter, zettel::frontmatter::DATE_FMT_STR};

    #[test]
    fn test_extract_from_string() {
        let test_suite: [(&'static str, (FrontMatter, &'static str)); 1] = [(
            r"---            
Title: LOL
Date: 2025-01-01 12:50:19 AM
Tags: whoa barber
---
",
            (
                FrontMatter::new(
                    "LOL",
                    DateTime::parse_from_str("2025-01-01 12:50:19 AM", DATE_FMT_STR).unwrap(),
                    vec!["whoa", "barber"],
                ),
                "",
            ),
        )];

        for (raw_text, (front_matter, remaining)) in &test_suite {
            let (extracted_front_matter, extracted_remaining) =
                FrontMatter::extract_from_str(*raw_text).unwrap();

            assert_eq!(extracted_front_matter, *front_matter);
            assert_eq!(extracted_remaining, *remaining);
        }
    }
}
