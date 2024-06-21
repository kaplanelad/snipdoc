//! This module provides cli command inject the snippets to the given
//! `inject_folder`
//!
//! The cli tool is collect all the snippets in the code, then load the snippets
//! in the DB (if given or if exists in the root path) and inject the snippets
//! in the placeholders.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use snipdoc::{
    cli::CmdExit,
    config::Config,
    db::{self, Db},
    parser::{
        collector::Collector,
        injector::{InjectedContent, Injector, InjectorResult},
    },
    walk,
};

use crate::Format;

/// Executes `snipdoc run` command
///
/// # Returns
///
/// This function returns a [`CmdExit`] indicating the success or failure
/// of the execution.
pub fn exec(
    config: &Config,
    inject_folder: &Path,
    db_file: Option<PathBuf>,
    dry_run: bool,
    format: &Format,
) -> CmdExit {
    let span = tracing::span!(tracing::Level::INFO, "run");
    let _guard = span.enter();
    let injector = match run(config, inject_folder, db_file) {
        Ok(i) => i,
        Err(err) => {
            return CmdExit::error_with_message(&format!("could not init walk instance: {err}"));
        }
    };

    if !dry_run {
        for (path, status) in injector.results.iter() {
            if let InjectedContent::Injected(summary) = status {
                write_content(path.as_path(), &summary.content).unwrap();
            }
        }
    }

    format.reporter().inject(inject_folder, &injector.results);

    CmdExit::ok()
}

pub fn run(
    config: &Config,
    inject_folder: &Path,
    db_file: Option<PathBuf>,
) -> io::Result<InjectorResult> {
    // first search a snippets from the code
    let walk = match walk::Walk::from_config(inject_folder, &config.walk) {
        Ok(walk) => walk,
        Err(err) => {
            return Err(err);
        }
    };

    let mut db_data = db::Code::new(Collector::walk(&walk).snippets)
        .load()
        .unwrap();

    // Then if db_file is given, load the snippets from the yaml,
    // If the db_file not given, search if the default snippet file name is
    // exists in the root folder. if true the snippets from the file.
    let maybe_yaml_file = {
        db_file.map_or_else(
            || db::Yaml::try_from_default_file(inject_folder),
            |db_file| Some(db::Yaml::new(db_file.as_path())),
        )
    };

    // If yaml db is configured, load all the snippets from the yaml and append to
    // the existing  snippets
    if let Some(yaml_db) = &maybe_yaml_file {
        let snippets_from_yaml = yaml_db.load().unwrap();
        db_data.snippets.extend(snippets_from_yaml.snippets);
        db_data.templates = snippets_from_yaml.templates;
        tracing::debug!(
            snippet_count = db_data.snippets.len(),
            template_count = db_data.templates.len(),
            "yaml file loaded successfully"
        );
    }

    let walk = match walk::Walk::from_config(inject_folder, &config.walk) {
        Ok(walk) => walk,
        Err(err) => {
            return Err(err);
        }
    };

    Ok(Injector::walk(&walk, &db_data, &config.inject))
}

fn write_content(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
