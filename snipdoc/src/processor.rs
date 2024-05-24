//! This module provides functionality for collecting and inject snippets from
//! files within a folder.

use std::{
    collections::{BTreeMap, HashMap},
    fs,
    fs::File,
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    db::{DBData, Snippet},
    parser::{collector::CollectSnippet, injector::InjectSummary, ParseFile},
    walk::Walk,
};

/// A struct for collecting snippets from files within a folder.
#[derive(Debug, Serialize, Deserialize)]
pub struct Collector {
    pub root_folder: PathBuf,
    pub snippets: BTreeMap<PathBuf, Vec<CollectSnippet>>,
}

/// Represents the inject status result
#[derive(Debug, Serialize, Deserialize)]
pub struct Injector {
    pub root_folder: PathBuf,
    pub results: BTreeMap<PathBuf, InjectResult>,
}

/// Represent the injector status result
#[derive(Debug, Serialize, Deserialize)]
pub enum InjectResult {
    /// When found placeholder snippet section.
    Injected(InjectSummary),
    /// When found a snippet collection but not found inject snippet with the
    /// same it to injector
    None,
    /// When error is encountered
    Error(String),
}

impl Collector {
    /// Constructs a `Collector` instance by collecting snippets from files
    /// within the provided `Walk`.
    #[must_use]
    pub fn on_files(walk: &Walk) -> Self {
        let snippets = walk
            .get_files()
            .par_iter()
            .flat_map(|path| Self::on_file(path.as_path()).map(|findings| (path.clone(), findings)))
            .collect::<BTreeMap<_, _>>();

        Self {
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
    fn on_file(path: &Path) -> Option<Vec<CollectSnippet>> {
        let span = tracing::info_span!("parse_file", path = %path.display());
        let _guard = span.enter();

        let input = read_file_content_if_utf8(path)?;

        match ParseFile::new(&input).parse() {
            Ok(findings) => {
                tracing::debug!("parsed successfully");
                Some(findings)
            }
            Err(err) => {
                tracing::debug!(err = %err, "could not parse the file");
                None
            }
        }
    }
}

impl Injector {
    /// Constructs a `Collector` instance by collecting snippets from files
    /// within the provided `Walk`.
    #[must_use]
    pub fn on_files(walk: &Walk, db_data: &DBData) -> Self {
        let mut snippets_from = HashMap::new();

        for (snippet_id, snippet_data) in &db_data.snippets {
            snippets_from.insert(snippet_id.clone(), snippet_data);
        }

        let results = walk
            .get_files()
            .par_iter()
            .filter_map(|path| {
                read_file_content_if_utf8(path).map(|content| {
                    let status = Self::inject(&content, &snippets_from);
                    (path.clone(), status)
                })
            })
            .collect::<BTreeMap<PathBuf, InjectResult>>();

        Self {
            root_folder: walk.folder.clone(),
            results,
        }
    }

    // Processes a single file and extracts injected snippets.
    ///
    /// # Returns
    ///
    /// Returns `Some` containing the collected snippets if successful,
    /// otherwise returns `None`.
    pub fn inject(input: &str, snippets: &HashMap<String, &Snippet>) -> InjectResult {
        match ParseFile::new(input).inject(snippets) {
            Ok(summary) => {
                if summary.actions.is_empty() {
                    tracing::debug!("not found inject content");
                    InjectResult::None
                } else {
                    tracing::debug!("content injected");
                    InjectResult::Injected(summary)
                }
            }
            Err(err) => {
                tracing::debug!(err = %err, "could not parse the given content");
                InjectResult::Error(err.to_string())
            }
        }
    }
}

fn is_utf8_file<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0; 4096];

    while let Ok(n) = reader.read(&mut buffer) {
        if n == 0 {
            break;
        }
        if std::str::from_utf8(&buffer[..n]).is_err() {
            return Ok(false);
        }
    }

    Ok(true)
}

fn read_file_content_if_utf8<P: AsRef<Path>>(path: P) -> Option<String> {
    match is_utf8_file(&path) {
        Ok(true) => match fs::read_to_string(path) {
            Ok(content) => Some(content),
            Err(err) => {
                tracing::debug!(err = %err, "could not read file content");
                None
            }
        },
        Ok(false) => {
            tracing::trace!("filter out non-UTF-8 files");
            None
        }
        Err(err) => {
            tracing::debug!(err = %err, "could not read file content");
            None
        }
    }
}
