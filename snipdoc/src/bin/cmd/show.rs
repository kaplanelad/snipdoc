//! This module provides cli command to preview all the snippets

use std::path::{Path, PathBuf};

use snipdoc::{
    cli::CmdExit,
    config::Config,
    db::{self, DBData, Db},
    parser::{collector::Collector, SnippetKind},
    walk,
};

use crate::Format;

/// Executes `snipdoc show` command
///
/// # Returns
///
/// This function returns a [`CmdExit`] indicating the success or failure
/// of the execution.
pub fn exec(
    config: &Config,
    inject_folder: &Path,
    snippet_kind: &SnippetKind,
    db_file: Option<PathBuf>,
    format: &Format,
) -> CmdExit {
    // collect first snippets from code
    let walk = match walk::Walk::from_config(inject_folder, &config.walk) {
        Ok(walk) => walk,
        Err(err) => {
            return CmdExit::error_with_message(&format!("could not init walk instance: {err}"));
        }
    };

    let mut snippets_data = DBData::default();

    if snippet_kind == &SnippetKind::Code || snippet_kind == &SnippetKind::Any {
        let code_snippets = db::Code::new(Collector::walk(&walk).snippets)
            .load()
            .unwrap();

        snippets_data.snippets.extend(code_snippets.snippets);
    }

    if snippet_kind == &SnippetKind::Yaml || snippet_kind == &SnippetKind::Any {
        let maybe_yaml_file = {
            db_file.map_or_else(
                || db::Yaml::try_from_default_file(inject_folder),
                |db_file| Some(db::Yaml::new(db_file.as_path())),
            )
        };

        if let Some(yaml_db) = &maybe_yaml_file {
            let snippets_from_yaml = yaml_db.load().unwrap();
            snippets_data.snippets.extend(snippets_from_yaml.snippets);
        }
    }

    format
        .reporter()
        .snippets(inject_folder, &snippets_data.snippets);

    CmdExit::ok()
}
