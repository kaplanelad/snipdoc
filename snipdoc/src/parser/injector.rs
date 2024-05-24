use std::{collections::HashMap, fmt::Write, hash::BuildHasher, str::FromStr};

use pest::iterators::Pairs;
use serde::{Deserialize, Serialize};

use super::{html_tag, Rule};
use crate::{
    db::{Snippet, SnippetKind},
    errors::ParserResult,
};

const INJECT_FROM_ATTRIBUTE_NAME: &str = "inject_from";
const STRIP_PREFIX_ATTRIBUTE_NAME: &str = "strip_prefix";
const ADD_PREFIX_ATTRIBUTE_NAME: &str = "add_prefix";

/// A struct representing the injection summary result.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InjectSummary {
    /// Hold all the content in the given input with the snip injection
    /// logic
    pub content: String,
    /// Represent the action that occurred.
    pub actions: Vec<InjectAction>,
}

/// The action which occurred
#[derive(Debug, Serialize, Deserialize)]
pub enum InjectAction {
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

            let snippet_id = attributes
                .get("id")
                .expect("assertion fails, snippet without element id");

            let maybe_inject_from = {
                attributes.get(INJECT_FROM_ATTRIBUTE_NAME).and_then(|k| {
                    SnippetKind::from_str(k).map_or_else(
                        |()| {
                            tracing::debug!(inject_from = k, "invalid inject_from kind.");
                            None
                        },
                        Some,
                    )
                })
            };

            if let Some(inject_from) = maybe_inject_from {
                if let Some(snippet) = snippets.get(snippet_id) {
                    if inject_from == SnippetKind::Any || inject_from == snippet.kind {
                        let snippet_content = snippet
                            .get_content(
                                attributes.get(STRIP_PREFIX_ATTRIBUTE_NAME),
                                attributes.get(ADD_PREFIX_ATTRIBUTE_NAME),
                            )
                            .join("\n");

                        let comment_tag = html_tag::get_comment_tag_open(&children);
                        let close_tag_of_tag_open =
                            html_tag::get_comment_tag_of_tag_open(&children);

                        let inject_result = format!(
                            "{comment_tag}{tag_open}{}{snippet_content}\n{tag_close}",
                            close_tag_of_tag_open.unwrap_or_default()
                        );

                        summary.content.write_str(&inject_result)?;

                        if pair.as_str() == inject_result {
                            summary.actions.push(InjectAction::Equal {
                                snippet_id: snippet_id.to_string(),
                            });
                        } else {
                            summary.actions.push(InjectAction::Injected {
                                snippet_id: snippet_id.to_string(),
                                content: snippet_content,
                            });
                        }
                    } else {
                        summary.content.write_str(pair.as_str())?;
                    }
                } else {
                    summary.actions.push(InjectAction::NotFound {
                        snippet_id: snippet_id.to_string(),
                        snippet_kind: inject_from,
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
