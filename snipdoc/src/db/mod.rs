mod code;
mod yaml;
use std::{collections::BTreeMap, path::PathBuf, str::FromStr};

pub use code::Code;
use serde::{Deserialize, Serialize};
pub use yaml::{Yaml, DEFAULT_FILE_NAME};

use crate::parser::collector::CollectSnippet;

/// A trait that defines the behavior for database operations.
pub trait Db {
    /// Loads data from the database.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the loaded `DBData` on success, or an [`Error`]
    /// on failure.
    ///
    /// # Errors
    ///
    /// Return and [`Error`] when could not load the data
    fn load(&self) -> Result<'_, DBData>;

    /// Saves a list of snippets to the database.
    ///
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Return and [`Error`] when could not save the data
    fn save(&self, snippets: &[&CollectSnippet]) -> Result<'_, ()>;
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct DBData {
    pub snippets: BTreeMap<String, Snippet>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Snippet {
    pub content: String,
    pub kind: SnippetKind,
    pub path: PathBuf,
}

#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SnippetKind {
    Yaml,
    Code,
    #[default]
    Any,
}

impl FromStr for SnippetKind {
    type Err = ();

    fn from_str(input: &str) -> std::result::Result<Self, ()> {
        match input {
            "yaml" => Ok(Self::Yaml),
            "code" => Ok(Self::Code),
            "any" => Ok(Self::Any),
            _ => Err(()),
        }
    }
}

impl Snippet {
    /// Returns the snippet content, filtered based on `strip_prefix` if
    /// specified.
    #[must_use]
    pub fn get_content(
        &self,
        strip_prefix: Option<&String>,
        add_prefix: Option<&String>,
    ) -> Vec<String> {
        self.content
            .split('\n')
            .filter_map(|line| {
                if line.contains("<snip") || line.contains("</snip") {
                    return None;
                }
                let line = strip_prefix.map_or_else(
                    || line.to_string(),
                    |prefix_inject| line.strip_prefix(prefix_inject).unwrap_or(line).to_string(),
                );

                if let Some(add_prefix) = add_prefix {
                    Some(format!("{add_prefix}{line}"))
                } else {
                    Some(line)
                }
            })
            .collect::<Vec<_>>()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    YAML(#[from] serde_yaml::Error),

    #[error("operation not supported")]
    NotSupported,
}

pub type Result<'a, T> = std::result::Result<T, Error>;
