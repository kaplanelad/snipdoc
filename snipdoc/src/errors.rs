//! A module for defining custom error types and result aliases for parsing
//!
//! This module provides custom error types for parsing and replacing operations
//! along with result aliases for convenient error handling.
use crate::parser::Rule;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    String(#[from] std::fmt::Error),

    #[error("{0}")]
    Pest(Box<pest::error::Error<Rule>>),

    #[error(transparent)]
    HtmlParsing(#[from] scraper::error::SelectorErrorKind<'static>),

    #[error("Selector `{selector}` not found in tag: {tag}")]
    SelectorNotFound { selector: String, tag: String },
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    YAMLFile(#[from] serde_yaml::Error),
}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(error: pest::error::Error<Rule>) -> Self {
        Self::Pest(Box::new(error))
    }
}

pub type ParserResult<'a, T> = std::result::Result<T, ParseError>;
pub type ConfigResult<'a, T> = std::result::Result<T, ConfigError>;
