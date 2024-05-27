pub mod collector;
pub mod html_tag;
pub mod injector;

use std::collections::HashMap;

use pest::Parser;
// use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;

use crate::{db::Snippet, errors::ParserResult};

#[derive(Parser)]
#[grammar = "snippet.pest"]
pub struct SnippetParse;

/// A structure representing a file to be parsed.
pub struct ParseFile<'a> {
    pub input: &'a str,
}

impl<'a> ParseFile<'a> {
    /// Constructs a new [`ParseFile`] with the provided input.
    #[must_use]
    pub const fn new(input: &'a str) -> Self {
        Self { input }
    }

    // /// Constructs a new [`ParseFile`] with the provided input.
    // #[must_use]
    // pub const fn new(input: &'a str) -> Self {
    //     Self { input }
    // }

    /// Parses the input file content and extracts snippets.
    ///
    /// # Errors
    ///
    /// This function may return an error if it fails to parse the input file.
    /// Other errors encountered during parsing will be logged.
    pub fn parse(&self) -> ParserResult<'_, Vec<collector::CollectSnippet>> {
        let pairs = SnippetParse::parse(Rule::file, self.input)?;

        let mut findings: Vec<collector::CollectSnippet> = vec![];
        collector::collect_snippets(pairs, &mut findings);
        Ok(findings)
    }

    /// Injects snippets in the input file content based on the provided
    /// [`Snippet`] map.
    ///
    /// # Errors
    ///
    /// This function may return an error if it fails to parse the input file.
    /// Other errors encountered during parsing will be logged.
    pub fn inject(
        &self,
        snippets: &HashMap<String, &Snippet>,
    ) -> ParserResult<'_, injector::InjectSummary> {
        let pairs = SnippetParse::parse(Rule::file, self.input)?;

        let mut inject_summary = injector::InjectSummary::default();
        injector::inject_snippets(pairs, &mut inject_summary, snippets)?;

        Ok(inject_summary)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;
    use crate::tests_cfg;

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

        let parser = ParseFile::new(content);
        assert_debug_snapshot!(parser.parse());
    }

    #[test]
    fn parsing_error() {
        let content = r#"<!-- <snip id="description"></snip"#;
        let parser: ParseFile = ParseFile::new(content);
        assert!(parser.parse().is_err());
    }

    #[test]
    fn get_inject() {
        let content = r#"# Snipdoc

<!-- <snip id="installation" inject_from="code"> -->
# inject `installation` snippet id from code snippet kind
<!-- </snip> -->

<!-- <snip id="inject_from_yaml" inject_from="yaml"> -->
# inject `inject_from_yaml` snippet id from yaml snippet kind
<!-- </snip> -->

<!-- <snip id="inject_from_yaml" inject_from="code"> -->
# Skip injection, `inject_from_yaml` snippet id not exists in code
<!-- </snip> -->

<!-- <snip id="inject_from_yaml" inject_from="any"> -->
# inject_from is any, and this id exists in the yaml
<!-- </snip> -->

<!-- <snip id="description" inject_from="code" add_prefix="//! "> -->
# Adding the prefix for each line
<!-- </snip> -->

<!-- <snip id="description" inject_from="code" strip_prefix="snip"> -->
# Strip `snip` word from prefix for each line
<!-- </snip> -->

<!-- <snip id="description" inject_from="code" template="```sh\n{snippet}\n```"> -->
# Add template to inject snippet
<!-- </snip> -->

<!-- <snip id="description" inject_from="code"> -->
snipdoc
<!-- </snip> -->

<!-- <snip id="not-found" inject_from="code"> -->
not-found
<!-- </snip> -->

"#;
        let parser: ParseFile = ParseFile::new(content);
        let snippets: HashMap<String, Snippet> = tests_cfg::get_snippet_to_inject();
        let snippet_refs: HashMap<String, &Snippet> =
            snippets.iter().map(|(k, v)| (k.clone(), v)).collect();

        assert_debug_snapshot!(parser.inject(&snippet_refs));
    }
}
