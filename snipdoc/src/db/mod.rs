mod code;
mod yaml;

use std::collections::BTreeMap;

pub use code::Code;
use serde::{Deserialize, Serialize};
pub use yaml::{Yaml, DEFAULT_FILE_NAME};

use crate::parser::{collector::CollectSnippet, Snippet};

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
