mod actions;
pub mod collector;
mod html_tag;
pub mod injector;

use std::{path::PathBuf, str::FromStr};

use pest_derive::Parser;
use serde::{Deserialize, Serialize};

#[cfg(feature = "exec")]
use crate::parser::actions::exec;

#[derive(Parser)]
#[grammar = "snippet.pest"]
pub struct SnippetParse;

#[derive(Serialize, Deserialize, Debug)]
pub struct Snippet {
    pub id: String,
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
    pub fn create_content(&self, inject_actions: &injector::InjectContentAction) -> String {
        #[cfg(feature = "exec")]
        let content = if inject_actions.kind == injector::InjectAction::Exec {
            exec::run(&self.content).unwrap_or_else(|err| {
                tracing::error!(err, "execute snippet command failed");
                self.content.to_string()
            })
        } else {
            self.content.to_string()
        };

        #[cfg(not(feature = "exec"))]
        let content = self.content.to_string();

        let content = inject_actions
            .template
            .before_inject(&content, &inject_actions.kind);

        let content = content
            .lines()
            .filter_map(|line| {
                // validate if i can remove this code
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
            .join(crate::LINE_ENDING);

        inject_actions
            .template
            .after_inject(&content, &inject_actions.kind)
    }
}

#[cfg(test)]
mod tests {
    use insta::{assert_debug_snapshot, with_settings};

    use super::*;
    use crate::{
        parser::injector::{InjectAction, Template},
        tests_cfg,
    };

    #[test]
    fn can_get_snippet_content_without_action() {
        let snippet = tests_cfg::get_snippet();

        let action = injector::InjectContentAction {
            kind: InjectAction::Copy,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: None,
            add_prefix: None,
            template: Template::default(),
        };

        with_settings!({filters => tests_cfg::redact::all()}, {
            assert_debug_snapshot!(snippet.create_content(&action));
        });
    }

    #[test]
    fn can_get_snippet_content_with_template_action() {
        let snippet = tests_cfg::get_snippet();

        let action = injector::InjectContentAction {
            kind: InjectAction::Copy,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: None,
            add_prefix: None,
            template: Template::new("```sh\n{snippet}\n```"),
        };

        with_settings!({filters => tests_cfg::redact::all()}, {
            assert_debug_snapshot!(snippet.create_content(&action));
        });
    }

    #[test]
    fn can_get_snippet_content_with_strip_prefix_action() {
        let snippet = tests_cfg::get_snippet();

        let action = injector::InjectContentAction {
            kind: InjectAction::Copy,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: Some("$ ".to_string()),
            add_prefix: None,
            template: Template::default(),
        };

        with_settings!({filters => tests_cfg::redact::all()}, {
            assert_debug_snapshot!(snippet.create_content(&action));
        });
    }

    #[test]
    fn can_get_snippet_content_with_add_prefix_action() {
        let snippet = tests_cfg::get_snippet();

        let action = injector::InjectContentAction {
            kind: InjectAction::Copy,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: None,
            add_prefix: Some("$".to_string()),
            template: Template::default(),
        };

        with_settings!({filters => tests_cfg::redact::all()}, {
            assert_debug_snapshot!(snippet.create_content(&action));
        });
    }

    #[test]
    fn can_get_snippet_content_with_combination_action() {
        let snippet = tests_cfg::get_snippet();

        let action = injector::InjectContentAction {
            kind: InjectAction::Copy,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: Some("$ ".to_string()),
            add_prefix: Some("- ".to_string()),
            template: Template::new("```sh\n{snippet}\n```"),
        };

        with_settings!({filters => tests_cfg::redact::all()}, {
            assert_debug_snapshot!(snippet.create_content(&action));
        });
    }

    #[cfg(feature = "exec")]
    #[test]
    fn can_get_snippet_with_exec_action_with_template() {
        let mut snippet = tests_cfg::get_snippet();
        snippet.content = r"echo calc result: $((1+1))".to_string();

        let action = injector::InjectContentAction {
            kind: InjectAction::Exec,
            snippet_id: "id".to_string(),
            inject_from: SnippetKind::Any,
            strip_prefix: None,
            add_prefix: None,
            template: Template::new("```sh\n{snippet}\n```"),
        };

        with_settings!({filters => tests_cfg::redact::all()}, {
            assert_debug_snapshot!(snippet.create_content(&action));
        });
    }
}
