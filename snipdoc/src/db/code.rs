//! This module defines the `Code` database, which get list collected snippets
//! from live code and converts them to a common structure for use with
//! `snipdoc`.
use std::{collections::BTreeMap, path::PathBuf};

use super::{DBData, Db, Error, Result, Snippet};
use crate::parser::{collector::CollectSnippet, SnippetKind};

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
                            id: snippet.id.clone(),
                            content: snippet.snippet.join(crate::LINE_ENDING),
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

#[cfg(test)]
mod tests {
    use insta::{assert_debug_snapshot, with_settings};

    use super::*;
    use crate::tests_cfg;

    #[test]
    fn can_load() {
        let code = Code {
            snippets: BTreeMap::from([(
                PathBuf::from("README.md"),
                tests_cfg::get_collect_snippets(),
            )]),
        };

        with_settings!({filters => vec![
            (tests_cfg::REGEX_REPLACE_LINE_ENDING, "\n")
        ]}, {
            assert_debug_snapshot!(code.load());
        });
    }

    #[test]
    fn can_save() {
        let code = Code {
            snippets: BTreeMap::new(),
        };

        let save_snippets = CollectSnippet {
            id: "description".to_string(),
            snippet: vec!["test".to_string(), "snipdoc".to_string()],
            inject_from: None,
            tag_open: "<snip id=\"description\">".to_string(),
            tag_close: "<!-- </snip> -->\n".to_string(),
        };

        with_settings!({filters => vec![
            (tests_cfg::REGEX_REPLACE_LINE_ENDING, "\n")
        ]}, {
            assert!(code.save(&[&save_snippets]).is_err());
        });
    }
}
