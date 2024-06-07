//! This module defines the local yaml database
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{DBData, Db, Result, Snippet};
use crate::parser::{collector::CollectSnippet, SnippetKind};

pub const DEFAULT_FILE_NAME: &str = "snipdoc.yml";

#[derive(Serialize, Deserialize)]
struct YamlSnippet {
    pub content: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
struct Data {
    pub snippets: BTreeMap<String, YamlSnippet>,
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
    fn save(&self, snippets: &[&CollectSnippet]) -> Result<'_, ()> {
        let mut data = Data::default();

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
    use insta::{assert_debug_snapshot, with_settings};

    use super::*;
    use crate::tests_cfg;

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

    #[test]
    fn can_save() {
        let root_folder = tree_fs::Tree::default().root_folder;
        let db_file_path = root_folder.join(DEFAULT_FILE_NAME);
        let yaml = Yaml::new(db_file_path.as_path());
        let snippets = tests_cfg::get_collect_snippets();
        let snippet_refs: Vec<&CollectSnippet> = snippets.iter().collect();
        assert!(yaml.save(&snippet_refs).is_ok());

        #[cfg(windows)]
        let replace_path = root_folder.display().to_string().replace(r"\", r"\\");

        #[cfg(not(windows))]
        let replace_path = root_folder.display().to_string();

        with_settings!({filters => {
            let mut clean = tests_cfg::redact::all();
            clean.push((replace_path.as_str(), "[PATH]"));
            clean
        }}, {
            assert_debug_snapshot!(std::fs::read_to_string(db_file_path));
        });
    }
}
