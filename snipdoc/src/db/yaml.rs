//! This module defines the local yaml database
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{DBData, Db, Result, Snippet};
use crate::parser::{collector::CollectSnippet, SnippetKind, SnippetTemplate};

pub const DEFAULT_FILE_NAME: &str = "snipdoc.yml";

#[derive(Serialize, Deserialize)]
struct YamlSnippet {
    pub content: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
struct Data {
    #[serde(default)]
    pub snippets: BTreeMap<String, YamlSnippet>,
    #[serde(default)]
    pub templates: BTreeMap<String, SnippetTemplate>,
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
            tracing::debug!("{DEFAULT_FILE_NAME} file found");
            Some(Self::new(maybe_file.as_path()))
        } else {
            tracing::debug!("{DEFAULT_FILE_NAME} file not found");
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

        let mut data = DBData {
            snippets: BTreeMap::new(),
            templates: yaml_data.templates,
        };
        for (id, snippet) in &yaml_data.snippets {
            data.snippets.insert(
                id.clone(),
                Snippet {
                    id: id.clone(),
                    content: snippet.content.clone(),
                    kind: SnippetKind::Yaml,
                    path: snippet.path.clone(),
                },
            );
        }

        Ok(data)
    }

    /// Save snippets into a yaml file
    fn save(
        &self,
        snippets: &[&CollectSnippet],
        templates: &BTreeMap<String, SnippetTemplate>,
    ) -> Result<'_, ()> {
        let mut data = Data {
            snippets: BTreeMap::new(),
            templates: templates.clone(),
        };

        for snippet in snippets {
            // we should save only snippets and not the placeholders
            if snippet.inject_from.is_none() {
                data.snippets.insert(
                    snippet.id.to_string(),
                    YamlSnippet {
                        content: snippet.snippet.join(crate::LINE_ENDING),
                        path: self.path.clone(),
                    },
                );
            }
        }

        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let mut file: File = File::create(&self.path)?;
        file.write_all(serde_yaml::to_string(&data)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    #[cfg(not(windows))]
    use insta::with_settings;

    use super::*;
    #[cfg(not(windows))]
    use crate::tests_cfg;

    use crate::db::EMPTY_TEMPLATE_SNIPPETS;

    #[test]
    fn can_load() {
        let yaml_content = r"
        files:
        - path: snipdoc-snippets.yaml
          content: |
            snippets:
                one:
                    content: |2-
                        THIS SNIPPET FROM YAML FILE
                        <!-- </snip> -->
                    path: ./snipdoc-snippets.yaml
                two:
                    content: two
                    path: ./snipdoc-snippets.yaml
        ";

        let path = tree_fs::from_yaml_str(yaml_content).unwrap();
        let yaml_db = Yaml::new(path.join("snipdoc-snippets.yaml").as_path());
        assert_debug_snapshot!(yaml_db.load());
    }

    #[test]
    fn try_load_from_default_file() {
        let yaml_content = r"
        files:
        - path: snipdoc.yml
          content: 
        ";

        let path = tree_fs::from_yaml_str(yaml_content).unwrap();
        // when file exists
        assert!(Yaml::try_from_default_file(path.as_path()).is_some());

        // when default if not found
        assert!(Yaml::try_from_default_file(Path::new("path")).is_none());
    }

    #[cfg(not(windows))]
    #[test]
    fn can_save() {
        let root_folder = tree_fs::Tree::default().root_folder;
        let db_file_path = root_folder.join(DEFAULT_FILE_NAME);
        let yaml = Yaml::new(db_file_path.as_path());
        let snippets = tests_cfg::get_collect_snippets();
        let snippet_refs: Vec<&CollectSnippet> = snippets.iter().collect();
        assert!(yaml.save(&snippet_refs, &EMPTY_TEMPLATE_SNIPPETS).is_ok());
        with_settings!({filters => {
           vec![(root_folder.display().to_string().as_str(), "[PATH]")]
        }}, {
            assert_debug_snapshot!(std::fs::read_to_string(db_file_path));
        });
    }
}
