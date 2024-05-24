//! This module defines the local yaml database
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use super::{DBData, Db, Result, Snippet, SnippetKind};
use crate::parser::collector::CollectSnippet;

pub const DEFAULT_FILE_NAME: &str = "snipdoc.yml";

pub struct Yaml {
    pub path: PathBuf,
}

impl Yaml {
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    #[must_use]
    pub fn try_from_default_file(path: &Path) -> Option<Self> {
        let maybe_file = path.join(DEFAULT_FILE_NAME);
        if maybe_file.exists() {
            Some(Self::new(maybe_file.as_path()))
        } else {
            None
        }
    }
}

impl Db for Yaml {
    /// Load yaml snippets file to `DBData`
    ///
    /// # Errors
    ///
    /// return an error if the file not exists or not in the same schema.
    fn load(&self) -> Result<'_, DBData> {
        Ok(serde_yaml::from_reader(std::fs::File::open(&self.path)?)?)
    }

    /// Save snippets into a yaml file
    fn save(&self, snippets: &[&CollectSnippet]) -> Result<'_, ()> {
        let mut data = DBData::default();

        for snippet in snippets {
            // we should save only snippets and not the placeholders
            if snippet.inject_from.is_none() {
                data.snippets.insert(
                    snippet.id.to_string(),
                    Snippet {
                        content: snippet.snippet.join("\n"),
                        kind: SnippetKind::Yaml,
                        path: self.path.clone(),
                    },
                );
            }
        }

        let mut file: File = File::create(&self.path)?;
        file.write_all(serde_yaml::to_string(&data)?.as_bytes())?;
        Ok(())
    }
}
