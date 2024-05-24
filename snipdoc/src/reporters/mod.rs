pub mod console;
pub mod table;
use std::{collections::BTreeMap, path::Path};

use crate::{
    db::Snippet,
    processor::{InjectResults, InjectStats},
};

pub trait ReporterOutput: Sync {
    fn snippets(&self, root_folder: &Path, snippets: &BTreeMap<String, Snippet>);
    fn inject(&self, root_folder: &Path, result: &InjectResults);
    fn check(&self, _root_folder: &Path, _result: &InjectStats) {}
}
