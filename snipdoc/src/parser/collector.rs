use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    str::FromStr,
};

use pest::{iterators::Pairs, Parser};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::{html_tag, Rule};
use crate::{
    errors::ParserResult,
    parser::{SnippetKind, SnippetParse},
    read_file::RFile,
    walk::Walk,
};

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

pub struct Collector<'a> {
    pub input: &'a str,
}

/// A struct for collecting snippets from files within a folder.
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectSnippetsResults {
    pub root_folder: PathBuf,
    pub snippets: BTreeMap<PathBuf, Vec<CollectSnippet>>,
}

impl<'a> Collector<'a> {
    /// Constructs a new [`ParseFile`] with the provided input.
    #[must_use]
    pub const fn new(input: &'a str) -> Self {
        Self { input }
    }

    /// Constructs a `Collector` instance by collecting snippets from files
    /// within the provided `Walk`.
    #[must_use]
    pub fn walk(walk: &Walk) -> CollectSnippetsResults {
        let files = walk.get_files();

        tracing::debug!(
            count_files = files.len(),
            "start collect snippets from code"
        );
        let snippets = files
            .par_iter()
            .flat_map(|path| Self::file(path.as_path()).map(|findings| (path.clone(), findings)))
            .collect::<BTreeMap<_, _>>();

        CollectSnippetsResults {
            root_folder: walk.folder.clone(),
            snippets,
        }
    }

    // Processes a single file and extracts [`Vec<CollectSnippet>`].
    ///
    /// # Returns
    ///
    /// Returns `Some` containing the collected snippets if successful,
    /// otherwise returns `None`.
    #[allow(clippy::missing_errors_doc)]
    pub fn file(path: &Path) -> ParserResult<'_, Vec<CollectSnippet>> {
        let span = tracing::info_span!("parse_file", path = %path.display());
        let _guard = span.enter();

        let r_file = RFile::new(path)?;

        Collector::new(&r_file.content).run()
    }

    /// Parses the input file content and extracts snippets.
    ///
    /// # Errors
    ///
    /// This function may return an error if it fails to parse the input file.
    /// Other errors encountered during parsing will be logged.
    fn run(&self) -> ParserResult<'_, Vec<CollectSnippet>> {
        let pairs = SnippetParse::parse(Rule::file, self.input)?;

        let mut findings: Vec<CollectSnippet> = vec![];
        Self::collect_snippets(pairs, &mut findings);
        Ok(findings)
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
    fn collect_snippets(pairs: Pairs<'_, Rule>, snippets: &mut Vec<CollectSnippet>) {
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

                    let attributes = match html_tag::get_tag_attributes(tag_open) {
                        Ok(attributes) => attributes,
                        Err(err) => {
                            tracing::debug!(tag_open, err = %err, "could not extract attributes from the tag");
                            continue;
                        }
                    };

                    tracing::debug!(
                        tag_open,
                        tag_close,
                        attributes = format!("{:#?}", attributes),
                        "found attributes"
                    );

                    let mut lines = pair
                        .as_str()
                        .lines()
                        .map(std::string::ToString::to_string)
                        .skip(1)
                        .collect::<Vec<_>>();

                    lines.pop();

                    if let Some(last) = lines.last() {
                        if last == &tag_close.replace('\n', "") {
                            // TODO:: replace \n to crate::LINE_ENDING
                            lines.pop();
                        }
                    }

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

                    Self::collect_snippets(children, snippets);
                }
                _ => {
                    Self::collect_snippets(inner.clone(), snippets);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn can_collect() {
        let content = r#"# Snipdoc
# parse the snippet with `inject_from` attribute
<!-- <snip id="description" inject_from="code"> -->
<!-- </snip> -->

# parse snippet with html comment
<!-- <snip id="installation"> -->
$ cargo install snipdoc
$ ssnipdoc --version
<!-- </snip> -->

# parse snippet without any spaces between the comment
<!--<snip id="no-spaces">-->
$ cargo install snipdoc
$ ssnipdoc --version
<!--</snip>-->

# parse snippet with double slash comment
// <snip id="double-slash">
double-slash
// </snip>

# parse snippet with triple slash comment
/// <snip id="triple-slash">
triple-slash
/// </snip>
///
# parse snippet with triple hashtag comment
# <snip id="hashtag">
hashtag
# </snip>

# Inner snippets
<!-- <snip id="level-1" -->
Level 1
// <snip id="level-2">
Level 2
// <snip id="level-3">
Level 3
// </snip>
// </snip>
<!-- </snip> -->
"#;

        let collector = Collector::new(content);
        assert_debug_snapshot!(collector.run());
    }
}
