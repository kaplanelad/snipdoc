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
    db::{self, Db},
    processor::{Collector, Injector},
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
    inject_folder: &Path,
    db_file: Option<PathBuf>,
    dry_run: bool,
    format: &Format,
) -> CmdExit {
    let injector = match run(inject_folder, db_file) {
        Ok(i) => i,
        Err(err) => {
            return CmdExit::error_with_message(&format!("could not init walk instance: {err}"));
        }
    };

    if !dry_run {
        for (path, status) in injector.results.iter() {
            if let snipdoc::processor::InjectContentResult::Injected(summary) = status {
                write_content(path.as_path(), &summary.content).unwrap();
            }
        }
    }

    format.reporter().inject(inject_folder, &injector.results);

    CmdExit::ok()
}

pub fn run(inject_folder: &Path, db_file: Option<PathBuf>) -> io::Result<Injector> {
    // first search a snippets from the code
    let walk = match walk::Walk::new(inject_folder) {
        Ok(walk) => walk,
        Err(err) => {
            return Err(err);
        }
    };

    let mut snippets = db::Code::new(Collector::on_files(&walk).snippets)
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
        snippets.snippets.extend(snippets_from_yaml.snippets);
    }

    let config = {
        let mut config = walk::Config::try_from_default_file(inject_folder);
        if let Some(snipdocs_file) = maybe_yaml_file {
            config.excludes_file_path.push(snipdocs_file.path);
        }
        config
    };

    let walk = match walk::Walk::from_config(inject_folder, &config) {
        Ok(walk) => walk,
        Err(err) => {
            return Err(err);
        }
    };

    Ok(Injector::on_files(&walk, &snippets))
}

fn write_content(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
