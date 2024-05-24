use std::collections::BTreeMap;

use pest::iterators::Pairs;
use scraper::{Html, Selector};

use super::Rule;
use crate::errors::{ParseError, ParserResult};

/// Extracts the tag open element from the captured pair.
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
#[must_use]
pub fn get_tag_open<'b>(pair_children: &'b Pairs<'_, Rule>) -> &'b str {
    let pair = pair_children
        .clone()
        .next()
        .expect("assertion fails, snippet without tag open");

    assert!(
        pair.as_rule() == Rule::tag_open,
        "Expected tag_open rule, found {:?}",
        pair.as_rule()
    );

    pair.into_inner()
        .nth(1)
        .expect("Expected at least two")
        .as_str()
}

/// Extracts the tag close element from the captured pair.
///
///
/// This function iterates over the provided `pair_children` iterator to
/// find the tag close element. It assumes that the captured pairs
/// represent a snippet, and it searches for the tag close element
/// within these pairs.
///
/// # Panics
///
/// This function panics if the tag close element is not found in the
/// captured pairs. In production code, encountering this panic
/// indicates a severe problem, such as a misconfiguration or a critical
/// issue in the parser's behavior, which requires immediate attention.
/// In testing scenarios, this panic should be captured to ensure the
/// correctness of the parser.
#[must_use]
pub fn get_tag_close(pair_children: Pairs<'_, Rule>) -> &'_ str {
    for x in pair_children {
        match x.as_rule() {
            Rule::tag_close => {
                return x.as_str();
            }
            _ => continue,
        }
    }
    panic!("tag close not found")
}

/// Extracts the attributes from the given HTML tag.
///
/// For Example:
/// * The given tag is `<snip id="quick_start">` the result will be {"id":
///   "`quick_start`"}
///
/// # Errors
///
/// This function returns an error in the following cases:
///
/// * The tag is not of type `snipdoc`.
/// * The tag cannot be parsed.
pub fn get_tag_attributes(tag: &str) -> ParserResult<'_, BTreeMap<String, String>> {
    let html = Html::parse_fragment(tag);

    let selector_name = "snip";
    let selector = Selector::parse(selector_name)?;
    let attributes = html
        .select(&selector)
        .next()
        .ok_or_else(|| ParseError::SelectorNotFound {
            selector: selector_name.to_string(),
            tag: tag.to_string(),
        })?
        .value()
        .attrs();

    Ok(attributes
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect())
}

/// Extracts the comment tag from the captured pair.
///
/// for the given tag: <!-- <snip id="SNIPPET_ID"> the results will be result of
/// this function is `<!--`
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
#[must_use]
pub fn get_comment_tag_open<'b>(pair_children: &'b Pairs<'_, Rule>) -> &'b str {
    let pair = pair_children
        .clone()
        .next()
        .expect("assertion fails, snippet without tag open");

    assert!(
        pair.as_rule() == Rule::tag_open,
        "Expected tag_open rule, found {:?}",
        pair.as_rule()
    );

    pair.into_inner()
        .next()
        .expect("Expected at least two")
        .as_str()
}

/// Extracts the close comment tag from the captured pair.
///
/// for the given tag: <!-- <snip id="SNIPPET_ID"> --> the results will be
/// result of this function is `-->`
///
///
/// # Panics
///
/// This function assumes that the parsing configuration always captures a
/// snippet containing the  comment close of the tag. If this assumption is
/// violated, it indicates a misconfiguration or a critical issue in the
/// parser's behavior. Consequently, in production code, encountering this panic
/// indicates a severe problem that requires immediate attention.
/// In testing scenarios, this panic should be captured to ensure the
/// correctness of the parser.
#[must_use]
pub fn get_comment_tag_of_tag_open<'b>(pair_children: &'b Pairs<'_, Rule>) -> Option<&'b str> {
    let pair = pair_children
        .clone()
        .next()
        .expect("assertion fails, snippet without tag open");

    assert!(
        pair.as_rule() == Rule::tag_open,
        "Expected tag_open rule, found {:?}",
        pair.as_rule()
    );

    pair.into_inner().nth(2).map(|p| p.as_str())
}
