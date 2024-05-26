//! This module provides functionality for collecting and inject snippets from
//! files within a folder.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    db::{DBData, Snippet},
    errors::ParserResult,
    parser::{
        collector::CollectSnippet,
        injector::{InjectAction, InjectSummary},
        ParseFile,
    },
    walk::Walk,
};

/// A struct for collecting snippets from files within a folder.
#[derive(Debug, Serialize, Deserialize)]
pub struct Collector {
    pub root_folder: PathBuf,
    pub snippets: BTreeMap<PathBuf, Vec<CollectSnippet>>,
}

#[derive(Default)]
pub struct InjectStats {
    pub equals: u64,
    pub injects: u64,
    pub inject_unique_files: HashSet<PathBuf>,
    pub errors: BTreeMap<PathBuf, String>,
    pub not_found: BTreeMap<PathBuf, HashSet<String>>,
    pub not_found_count: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct InjectResults(BTreeMap<PathBuf, InjectContentResult>);

impl InjectResults {
    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &InjectContentResult)> {
        self.0.iter()
    }

    #[must_use]
    pub fn stats(&self) -> InjectStats {
        let mut stats = InjectStats::default();

        for (file, status) in self.iter() {
            match status {
                InjectContentResult::Injected(summary) => {
                    for action in &summary.actions {
                        match action {
                            InjectAction::Equal { .. } => stats.equals += 1,
                            InjectAction::Injected { .. } => {
                                stats.injects += 1;
                                stats.inject_unique_files.insert(file.clone());
                            }
                            InjectAction::NotFound { snippet_id, .. } => {
                                stats
                                    .not_found
                                    .entry(file.clone())
                                    .or_insert_with(|| HashSet::from([snippet_id.to_string()]))
                                    .insert(snippet_id.to_string());
                                stats.not_found_count += 1;
                            }
                        }
                    }
                }
                InjectContentResult::None => (),
                InjectContentResult::Error(err) => {
                    stats.errors.insert(file.clone(), err.to_string());
                }
            }
        }

        stats
    }
}

/// Represents the inject status result
#[derive(Debug, Serialize, Deserialize)]
pub struct Injector {
    pub root_folder: PathBuf,
    pub results: InjectResults,
}

/// Represent the injector status result
#[derive(Debug, Serialize, Deserialize)]
pub enum InjectContentResult {
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
    fn on_file(path: &Path) -> ParserResult<Vec<CollectSnippet>> {
        let span = tracing::info_span!("parse_file", path = %path.display());
        let _guard = span.enter();

        let input = read_file(path)?;

        ParseFile::new(&input).parse()
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
            .filter_map(|path| match read_file(path) {
                Ok(content) => {
                    let status = Self::inject(&content, &snippets_from);
                    Some((path.clone(), status))
                }
                Err(_err) => None,
            })
            .collect::<BTreeMap<PathBuf, InjectContentResult>>();

        Self {
            root_folder: walk.folder.clone(),
            results: InjectResults(results),
        }
    }

    // Processes a single file and extracts injected snippets.
    ///
    /// # Returns
    ///
    /// Returns `Some` containing the collected snippets if successful,
    /// otherwise returns `None`.
    pub fn inject(input: &str, snippets: &HashMap<String, &Snippet>) -> InjectContentResult {
        match ParseFile::new(input).inject(snippets) {
            Ok(summary) => {
                if summary.actions.is_empty() {
                    tracing::debug!("not found inject content");
                    InjectContentResult::None
                } else {
                    tracing::debug!("content injected");
                    InjectContentResult::Injected(summary)
                }
            }
            Err(err) => {
                tracing::debug!(err = %err, "could not parse the given content");
                InjectContentResult::Error(err.to_string())
            }
        }
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> ParserResult<'static, String> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}
