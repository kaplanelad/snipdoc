use std::str::FromStr;

use pest::iterators::Pairs;
use serde::{Deserialize, Serialize};

use super::{html_tag, Rule};
use crate::db::SnippetKind;

/// A struct representing a snippet extracted from code
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectSnippet {
    /// ID of the snippet.
    pub id: String,
    /// Defined if `inject` attribute exists in the snippet.
    pub inject_from: Option<SnippetKind>,
    /// Collect if `strip_prefix` attribute if exists exists in the snippet.
    // pub strip_prefix: Option<String>,
    /// Defined the tag open value of the snippet.
    pub tag_open: String,
    /// Defined the the tag close value of the snippet.
    pub tag_close: String,
    /// Hold all the line content inside the snippet.
    pub snippet: Vec<String>,
}

/// Recursively collects snippets from the provided pairs and populates the
/// given vector with the snippets.
///
/// This function recursively traverses the pairs, extracting snippets and
/// their attributes.
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
pub fn collect_snippets(pairs: Pairs<'_, Rule>, snippets: &mut Vec<CollectSnippet>) {
    if pairs.len() == 0 {
        return;
    }

    for pair in pairs {
        let inner = pair.clone().into_inner();

        match pair.as_rule() {
            Rule::snippet => {
                let children: Pairs<'_, Rule> = pair.clone().into_inner();
                let tag_open = html_tag::get_tag_open(&children);
                let tag_close = html_tag::get_tag_close(children.clone());

                tracing::debug!(tag_open, "found open tag");
                let attributes = match html_tag::get_tag_attributes(tag_open) {
                    Ok(attributes) => attributes,
                    Err(err) => {
                        tracing::debug!(tag_open, err = %err, "could not extract attributes from the tag");
                        continue;
                    }
                };

                tracing::debug!(
                    tag_open,
                    attributes = format!("{:#?}", attributes),
                    "found attributes"
                );

                let mut lines = children
                    .clone()
                    .nth(1)
                    // the parsing configuration always captures a snippet content. If indicates a
                    // misconfiguration or a critical issue in the parser's behavior. Consequently,
                    // in production code, encountering this panic indicates a severe problem that
                    // requires immediate attention. this assumption is In testing scenarios, this
                    // panic should be captured to ensure the correctness of the parser.
                    // violated, it
                    .unwrap()
                    .as_str()
                    .split('\n')
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>();

                lines.pop();

                snippets.push(CollectSnippet {
                    // Attribute ID as part of the parser configuration is
                    // mandatory. the snippet should't be captured if id
                    // element is not present. In this case
                    // user `expect` should brake the parser.
                    id: attributes
                        .get("id")
                        .expect("assertion fails, snippet without element id")
                        .to_string(),
                    inject_from: attributes
                        .get("inject_from")
                        .and_then(|k| SnippetKind::from_str(k).ok()),
                    tag_open: tag_open.to_string(),
                    tag_close: tag_close.to_string(),
                    snippet: lines,
                });

                collect_snippets(children, snippets);
            }
            _ => {
                collect_snippets(inner.clone(), snippets);
            }
        }
    }
}
