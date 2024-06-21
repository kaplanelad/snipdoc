//! This module provides a CLI command to validate that all snippets are valid
//! and match the current injected versions. It is useful for incorporating into
//! CI workflows to ensure documentation accuracy and consistency.

use std::path::{Path, PathBuf};

use snipdoc::{cli::CmdExit, config::Config};

use super::{super::Format, run::run};

/// Executes `snipdoc check` command
///
/// # Returns
///
/// This function returns a [`CmdExit`] indicating the success or failure
/// of the execution.
pub fn exec(config: &Config, inject_folder: &Path, db_file: Option<PathBuf>) -> CmdExit {
    let span = tracing::span!(tracing::Level::INFO, "checks");
    let _guard = span.enter();
    let injector = match run(config, inject_folder, db_file) {
        Ok(i) => i,
        Err(err) => {
            return CmdExit::error_with_message(&format!("could not init walk instance: {err}"));
        }
    };

    let stats = injector.results.stats();
    Format::Console.reporter().check(inject_folder, &stats);

    if !stats.errors.is_empty() || stats.injects > 0 || stats.not_found_count > 0 {
        CmdExit::error()
    } else {
        CmdExit::ok()
    }
}
