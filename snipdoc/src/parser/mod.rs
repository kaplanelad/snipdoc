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
