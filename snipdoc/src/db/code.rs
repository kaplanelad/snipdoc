//! This module defines the `Code` database, which get list collected snippets
//! from live code and converts them to a common structure for use with
//! `snipdoc`.
use std::{collections::BTreeMap, path::PathBuf};

use super::{DBData, Db, Error, Result, Snippet, SnippetKind};
use crate::parser::collector::CollectSnippet;

pub struct Code {
    pub snippets: BTreeMap<PathBuf, Vec<CollectSnippet>>,
}

impl Code {
    #[must_use]
    pub fn new(snippets: BTreeMap<PathBuf, Vec<CollectSnippet>>) -> Self {
        Self { snippets }
    }
}

impl Db for Code {
    fn load(&self) -> Result<'_, DBData> {
        let mut data = DBData::default();

        for (path, snippets) in &self.snippets {
            for snippet in snippets {
                // when loading snippets from the code, we should get only snippets and not
                // placeholder that we should inject
                if snippet.inject_from.is_none() {
                    data.snippets.insert(
                        snippet.id.clone(),
                        Snippet {
                            content: snippet.snippet.join("\n"),
                            kind: SnippetKind::Code,
                            path: path.clone(),
                        },
                    );
                }
            }
        }

        Ok(data)
    }
    fn save(&self, _: &[&CollectSnippet]) -> Result<'_, ()> {
        Err(Error::NotSupported)
    }
}
