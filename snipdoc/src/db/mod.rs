mod code;
mod yaml;

cfg_if::cfg_if! {
    if #[cfg(feature = "exec")] {
        use std::env;
        use std::process::Command;
        use crate::parser::injector::InjectAction;
    } else {}

}

use std::{collections::BTreeMap, path::PathBuf, str::FromStr};

pub use code::Code;
use serde::{Deserialize, Serialize};
pub use yaml::{Yaml, DEFAULT_FILE_NAME};

use crate::parser::{collector::CollectSnippet, injector::InjectContentAction};

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
    pub fn get_content(&self, inject_actions: &InjectContentAction) -> Vec<String> {
        #[cfg(feature = "exec")]
        let content = if inject_actions.kind == InjectAction::Exec {
            if Self::approve_exec_command(&self.content) {
                tracing::debug!(
                    command = self.content,
                    "execute snippet
                content"
                );
                match Command::new("sh").arg("-c").arg(&self.content).output() {
                    Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
                    Err(e) => e.to_string(),
                }
            } else {
                self.content.to_string()
            }
        } else {
            self.content.to_string()
        };

        #[cfg(not(feature = "exec"))]
        let content = self.content.to_string();

        let content = inject_actions.template.as_ref().map_or_else(
            || content.to_string(),
            |template| {
                template
                    .get()
                    .replace("{snippet}", &content)
                    .replace("\\n", "\n")
            },
        );

        content
            .split('\n')
            .filter_map(|line| {
                if line.contains("<snip") || line.contains("</snip") {
                    return None;
                }
                let line = inject_actions.strip_prefix.as_ref().map_or_else(
                    || line.to_string(),
                    |prefix_inject| line.strip_prefix(prefix_inject).unwrap_or(line).to_string(),
                );

                if let Some(add_prefix) = &inject_actions.add_prefix {
                    Some(format!("{add_prefix}{line}"))
                } else {
                    Some(line)
                }
            })
            .collect::<Vec<_>>()
    }

    #[cfg(feature = "exec")]
    fn approve_exec_command(command: &str) -> bool {
        if env::var("SNIPDOC_SKIP_EXEC_COMMANDS").map_or(false, |val| val == "true") {
            true
        } else {
            let question = requestty::Question::confirm("confirm")
                .message(format!(
                    "Security Warning: Snipdoc is about to execute the following command: \
                     `{command}`. Do you approve?"
                ))
                .build();

            match requestty::prompt_one(question) {
                Ok(answer) => answer.as_bool().is_some_and(|a| a),
                Err(err) => {
                    tracing::debug!(err = %err, "prompt error");
                    false
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;
    use crate::{
        parser::injector::{InjectAction, Template},
        tests_cfg,
    };

    #[test]
    fn can_get_snippet_content_without_action() {
        let snippet = tests_cfg::get_snippet();

        let action = InjectContentAction {
            kind: InjectAction::Copy,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: None,
            add_prefix: None,
            template: None,
        };

        assert_debug_snapshot!(snippet.get_content(&action));
    }

    #[test]
    fn can_get_snippet_content_with_template_action() {
        let snippet = tests_cfg::get_snippet();

        let action = InjectContentAction {
            kind: InjectAction::Copy,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: None,
            add_prefix: None,
            template: Some(Template::new("```sh\n{snippet}\n```")),
        };
        assert_debug_snapshot!(snippet.get_content(&action));
    }

    #[test]
    fn can_get_snippet_content_with_strip_prefix_action() {
        let snippet = tests_cfg::get_snippet();

        let action = InjectContentAction {
            kind: InjectAction::Copy,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: Some("$ ".to_string()),
            add_prefix: None,
            template: None,
        };
        assert_debug_snapshot!(snippet.get_content(&action));
    }

    #[test]
    fn can_get_snippet_content_with_add_prefix_action() {
        let snippet = tests_cfg::get_snippet();

        let action = InjectContentAction {
            kind: InjectAction::Copy,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: None,
            add_prefix: Some("$".to_string()),
            template: None,
        };
        assert_debug_snapshot!(snippet.get_content(&action));
    }

    #[test]
    fn can_get_snippet_content_with_combination_action() {
        let snippet = tests_cfg::get_snippet();

        let action = InjectContentAction {
            kind: InjectAction::Copy,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: Some("$ ".to_string()),
            add_prefix: Some("- ".to_string()),
            template: Some(Template::new("```sh\n{snippet}\n```")),
        };
        assert_debug_snapshot!(snippet.get_content(&action));
    }

    #[cfg(feature = "exec")]
    #[test]
    fn can_get_snippet_with_exec_action_with_template() {
        let mut snippet = tests_cfg::get_snippet();
        snippet.content = r"echo calc result: $((1+1))".to_string();

        let action = InjectContentAction {
            kind: InjectAction::Exec,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: None,
            add_prefix: None,
            template: Some(Template::new("```sh\n{snippet}\n```")),
        };
        assert_debug_snapshot!(snippet.get_content(&action));
    }
}
