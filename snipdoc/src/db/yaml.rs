//! This module defines the local yaml database
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{DBData, Db, Result, Snippet, SnippetKind};
use crate::parser::collector::CollectSnippet;

pub const DEFAULT_FILE_NAME: &str = "snipdoc.yml";

#[derive(Serialize, Deserialize)]
struct YamlSnippet {
    pub content: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
struct Data {
    pub snippets: HashMap<String, YamlSnippet>,
}

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
        let yaml_data: Data = serde_yaml::from_reader(std::fs::File::open(&self.path)?)?;

        let mut data = DBData::default();
        for (id, snippet) in &yaml_data.snippets {
            data.snippets.insert(
                id.clone(),
                Snippet {
                    content: snippet.content.clone(),
                    kind: SnippetKind::Yaml,
                    path: snippet.path.clone(),
                },
            );
        }

        Ok(data)
    }

    /// Save snippets into a yaml file
    fn save(&self, snippets: &[&CollectSnippet]) -> Result<'_, ()> {
        let mut data = Data::default();

        for snippet in snippets {
            // we should save only snippets and not the placeholders
            if snippet.inject_from.is_none() {
                data.snippets.insert(
                    snippet.id.to_string(),
                    YamlSnippet {
                        content: snippet.snippet.join("\n"),
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
