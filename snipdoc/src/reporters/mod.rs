pub mod console;
pub mod table;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{db::Snippet, processor::InjectResult};

pub trait ReporterOutput: Sync {
    fn snippets(&self, root_folder: &Path, snippets: &BTreeMap<String, Snippet>);
    fn inject(&self, root_folder: &Path, result: &BTreeMap<PathBuf, InjectResult>);
}
