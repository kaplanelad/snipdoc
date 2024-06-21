mod code;
mod yaml;

use std::collections::BTreeMap;

use crate::parser::{collector::CollectSnippet, Snippet, SnippetTemplate};
pub use code::Code;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
pub use yaml::{Yaml, DEFAULT_FILE_NAME};

lazy_static! {
    pub static ref EMPTY_COLLECTED_SNIPPETS: Vec<CollectSnippet> = vec![CollectSnippet {
        id: "SNIPPET_ID".to_string(),
        inject_from: None,
        tag_open: String::new(),
        tag_close: String::new(),
        snippet: vec![String::new()],
    }];
    pub static ref EMPTY_TEMPLATE_SNIPPETS: BTreeMap<String, SnippetTemplate> = BTreeMap::from([(
        "TEMPLATE_ID".to_string(),
        SnippetTemplate {
            content: r"```sh
{snippet}
```"
            .to_string(),
        }
    )]);
}

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
    fn save(&self, _: &[&CollectSnippet], _: &BTreeMap<String, SnippetTemplate>) -> Result<'_, ()> {
        Err(Error::NotSupported)
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct DBData {
    pub snippets: BTreeMap<String, Snippet>,
    pub templates: BTreeMap<String, SnippetTemplate>,
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
