use std::{
    collections::{BTreeMap, HashMap},
    fmt::Write,
    hash::BuildHasher,
    str::FromStr,
};

use pest::iterators::Pairs;
use serde::{Deserialize, Serialize};

use super::{html_tag, Rule};
use crate::{
    db::{Snippet, SnippetKind},
    errors::ParserResult,
};

const INJECT_ACTION: &str = "action";
const INJECT_FROM_ATTRIBUTE_NAME: &str = "inject_from";
const STRIP_PREFIX_ATTRIBUTE_NAME: &str = "strip_prefix";
const ADD_PREFIX_ATTRIBUTE_NAME: &str = "add_prefix";
const ADD_TEMPLATE: &str = "template";

/// A struct representing the injection summary result.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InjectSummary {
    /// Hold all the content in the given input with the snip injection
    /// logic
    pub content: String,
    /// Represent the action that occurred.
    pub actions: Vec<InjectStatus>,
}

#[derive(PartialEq, Eq)]
pub enum InjectAction {
    Copy,
    #[cfg(feature = "exec")]
    Exec,
}

impl FromStr for InjectAction {
    type Err = ();

    fn from_str(input: &str) -> std::result::Result<Self, ()> {
        match input {
            "copy" => Ok(Self::Copy),
            #[cfg(feature = "exec")]
            "exec" => Ok(Self::Exec),
            _ => Err(()),
        }
    }
}

pub struct InjectContentAction {
    pub snippet_id: String,
    pub kind: InjectAction,
    pub inject_from: SnippetKind,
    pub strip_prefix: Option<String>,
    pub add_prefix: Option<String>,
    pub template: Option<Template>,
}

pub enum Template {
    Text,
    Json,
    Yaml,
    Toml,
    Html,
    Rust,
    Python,
    Go,
    Sql,
    Shell,
    Bash,
    Sh,
    Custom(String),
}

impl Template {
    #[must_use]
    pub fn new(s: &str) -> Self {
        match s {
            "text" => Self::Text,
            "json" => Self::Json,
            "yaml" => Self::Yaml,
            "toml" => Self::Toml,
            "html" => Self::Html,
            "rust" => Self::Rust,
            "python" => Self::Python,
            "go" => Self::Go,
            "sql" => Self::Sql,
            "shell" => Self::Shell,
            "bash" => Self::Bash,
            "sh" => Self::Sh,
            _ => Self::Custom(s.to_string()),
        }
    }

    #[must_use]
    pub fn get(&self) -> String {
        match self {
            Self::Text => r"```text\n{snippet}\n```".to_string(),
            Self::Json => r"```json\n{snippet}\n```".to_string(),
            Self::Yaml => r"```yaml\n{snippet}\n```".to_string(),
            Self::Toml => r"```toml\n{snippet}\n```".to_string(),
            Self::Html => r"```html\n{snippet}\n```".to_string(),
            Self::Rust => r"```rust\n{snippet}\n```".to_string(),
            Self::Python => r"```python\n{snippet}\n```".to_string(),
            Self::Go => r"```go\n{snippet}\n```".to_string(),
            Self::Sql => r"```sql\n{snippet}\n```".to_string(),
            Self::Shell => r"```shell\n{snippet}\n```".to_string(),
            Self::Bash => r"```bash\n{snippet}\n```".to_string(),
            Self::Sh => r"```sh\n{snippet}\n```".to_string(),
            Self::Custom(template) => template.clone(),
        }
    }
}

impl InjectContentAction {
    pub fn new(attributes: &BTreeMap<String, String>) -> Option<Self> {
        let snippet_id = attributes.get("id").or({
            tracing::debug!(
                attributes = format!("{:?}", attributes),
                "attribute id not found in the given attributes"
            );
            None
        })?;

        let inject_from = attributes.get(INJECT_FROM_ATTRIBUTE_NAME).or({
            tracing::debug!(
                attributes = format!("{:?}", attributes),
                "attribute inject_from not found in the given attributes"
            );
            None
        })?;

        let Ok(inject_from) = SnippetKind::from_str(inject_from) else {
            tracing::debug!(inject_from, "invalid inject_from kind.");
            return None;
        };

        Some(Self {
            snippet_id: snippet_id.to_string(),
            inject_from,
            strip_prefix: attributes.get(STRIP_PREFIX_ATTRIBUTE_NAME).cloned(),
            add_prefix: attributes.get(ADD_PREFIX_ATTRIBUTE_NAME).cloned(),
            template: attributes.get(ADD_TEMPLATE).map(|s| Template::new(s)),
            kind: attributes
                .get(INJECT_ACTION)
                .and_then(|a| InjectAction::from_str(a).ok())
                .unwrap_or(InjectAction::Copy),
        })
    }
}

/// The action which occurred
#[derive(Debug, Serialize, Deserialize)]
pub enum InjectStatus {
    /// The snippet found and contains the same content
    Equal { snippet_id: String },
    /// The snippet found and the content was injected
    Injected { snippet_id: String, content: String },
    /// When has injected the snippet but not found snippet
    NotFound {
        snippet_id: String,
        snippet_kind: SnippetKind,
    },
}

/// Injects snippets in the input file content based on the provided
/// `snippets` map.
///
/// # Errors
///
/// This function may return an error if it fails to parse the input file.
/// Other errors encountered during parsing will be logged.
///
/// # Panics
///
/// This function assumes that the parsing configuration always captures a
/// snippet containing a tag open. If this assumption is violated, it
/// indicates a misconfiguration or a critical issue in the parser's
/// behavior. Consequently, in production code, encountering this panic
/// indicates a severe problem that requires immediate attention.
/// In testing scenarios, this panic should be captured to ensure the
/// correctness of the parser.
pub fn inject_snippets<'a, S: BuildHasher>(
    pairs: Pairs<'a, Rule>,
    summary: &'a mut InjectSummary,
    snippets: &'a HashMap<String, &'a Snippet, S>,
) -> ParserResult<'a, ()> {
    if pairs.len() == 0 {
        return Ok(());
    }

    for pair in pairs {
        let inner = pair.clone().into_inner();

        if pair.as_rule() == Rule::snippet {
            let children: Pairs<'_, Rule> = pair.clone().into_inner();

            let tag_open = html_tag::get_tag_open(&children);
            let tag_close = html_tag::get_tag_close(children.clone());

            let attributes = match html_tag::get_tag_attributes(tag_open) {
                Ok(attributes) => attributes,
                Err(err) => {
                    tracing::debug!(tag_open, err = %err, "could not extract attributes from the tag");
                    continue;
                }
            };

            let inject_content_actions = InjectContentAction::new(&attributes);

            if let Some(inject_actions) = inject_content_actions {
                if let Some(snippet) = snippets.get(&inject_actions.snippet_id) {
                    if inject_actions.inject_from == SnippetKind::Any
                        || inject_actions.inject_from == snippet.kind
                    {
                        let snippet_content = snippet.get_content(&inject_actions).join("\n");

                        let comment_tag = html_tag::get_comment_tag_open(&children);
                        let close_tag_of_tag_open =
                            html_tag::get_comment_tag_of_tag_open(&children);

                        let inject_result = format!(
                            "{comment_tag}{tag_open}{}{snippet_content}\n{tag_close}",
                            close_tag_of_tag_open.unwrap_or_default()
                        );

                        summary.content.write_str(&inject_result)?;

                        if pair.as_str() == inject_result {
                            summary.actions.push(InjectStatus::Equal {
                                snippet_id: inject_actions.snippet_id.to_string(),
                            });
                        } else {
                            summary.actions.push(InjectStatus::Injected {
                                snippet_id: inject_actions.snippet_id.to_string(),
                                content: snippet_content,
                            });
                        }
                    } else {
                        // summary.actions.push(InjectStatus::NotFound {
                        //     snippet_id: inject_actions.snippet_id.to_string(),
                        //     snippet_kind: inject_actions.inject_from,
                        // });
                        summary.content.write_str(pair.as_str())?;
                    }
                } else {
                    summary.actions.push(InjectStatus::NotFound {
                        snippet_id: inject_actions.snippet_id.to_string(),
                        snippet_kind: inject_actions.inject_from,
                    });
                    summary.content.write_str(pair.as_str())?;
                }
            } else {
                summary.content.write_str(pair.as_str())?;
            }
        } else {
            inject_snippets(inner.clone(), summary, snippets)?;
            if inner.len() == 0 {
                summary.content.write_str(pair.as_str())?;
            }
        }
    }
    Ok(())
}
