//! This module provides cli command for generate a `snipdoc` DB via the code
//! snippets directly or generate a empty yaml DB.
//!
//! The command create the yaml DB with the following options:
//! - Directly from the code by adding
//! - If `--empty` is given, create a template snippets
//!
//! ## Examples
//!
//! 1. Generate an empty YAML file and manage all the code snippets manually by
//!    running the command:
//! ```
//!    snipdoc create-db --empty
//! ```
//! 2. Generate a YAML file based on your existing code snippets by running the
//!    command:
//! ```
//!  snipdoc create-db
//! ```

use std::path::Path;

use snipdoc::{
    cli::CmdExit,
    config::Config,
    db::{self, Db},
    parser::collector::CollectSnippet,
    processor::Collector,
    walk,
};

/// Executes `snipdoc create-db` command
///
/// # Returns
///
/// This function returns a [`CmdExit`] indicating the success or failure
/// of the execution.
pub fn exec(config: &Config, collect_folder: &Path, empty: bool) -> CmdExit {
    let span = tracing::span!(tracing::Level::INFO, "collect", empty);
    let _guard = span.enter();

    let db_file_path = collect_folder.join(db::DEFAULT_FILE_NAME);

    let result = if empty {
        let example_snippets = [CollectSnippet {
            id: "EXAMPLE_ID".to_string(),
            inject_from: None,
            tag_open: String::new(),
            tag_close: String::new(),
            snippet: vec![String::new()],
        }];

        let example_snippet_refs: Vec<&CollectSnippet> = example_snippets.iter().collect();

        db::Yaml::new(&db_file_path).save(&example_snippet_refs)
    } else {
        let walk = match walk::Walk::from_config(collect_folder, &config.walk) {
            Ok(walk) => walk,
            Err(err) => {
                return CmdExit::error_with_message(&format!(
                    "could not init walk instance: {err}"
                ));
            }
        };

        let collector = Collector::on_files(&walk);

        let all_snippets: Vec<&snipdoc::parser::collector::CollectSnippet> =
            collector.snippets.values().flatten().collect();

        db::Yaml::new(&db_file_path).save(&all_snippets)
    };

    if let Err(err) = result {
        CmdExit::error_with_message(&err.to_string())
    } else {
        CmdExit::ok_with_message(&format!(
            "Wrote db file in path: {}",
            db_file_path.display()
        ))
    }
}
