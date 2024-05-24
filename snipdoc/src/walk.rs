//! This module provides functionality for traversing directories and collecting
//! files, while respecting include and exclude patterns.
//!
//! It utilizes the `ignore` crate for directory traversal and `serde` for
//! serialization and deserialization of configuration patterns.
use std::{
    io,
    path::{Path, PathBuf},
    sync::mpsc,
};

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::errors::ConfigResult;

pub const DEFAULT_CONFIG_NAME: &str = "snipdoc-config.yml";

/// Represents a directory walker with include and exclude configurations.
#[derive(Debug)]
pub struct Walk {
    /// The base folder from which files are collected.
    pub folder: PathBuf,
    config: Config,
}

/// Configuration for file inclusion and exclusion patterns.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Patterns to include files.
    #[serde(with = "serde_regex", default)]
    pub includes: Vec<Regex>,
    /// Patterns to exclude files.
    #[serde(with = "serde_regex", default)]
    pub excludes: Vec<Regex>,
    /// Patterns to exclude files.
    #[serde(default)]
    pub excludes_file_path: Vec<PathBuf>,
}

impl Config {
    /// Search if [`DEFAULT_CONFIG_NAME`] configuration file exists under the
    /// given path. if exists, loading the configuration from the config
    /// file. if not load the default
    pub fn try_from_default_file(path: &Path) -> Self {
        let maybe_config_exists = path.join(DEFAULT_CONFIG_NAME);

        if maybe_config_exists.exists() {
            if let Ok(config) = Self::from_file(maybe_config_exists.as_path()) {
                return config;
            }
            tracing::error!(
                config = %maybe_config_exists.display(),
                "invalid config file content"
            );
        } else {
            tracing::debug!(
                config = %maybe_config_exists.display(),
                "default config file not found"
            );
        }
        Self::default()
    }

    /// Convert config file to [`Config`]
    ///
    /// # Errors
    ///
    /// invalid schema
    pub fn from_file(path: &Path) -> ConfigResult<'_, Self> {
        Ok(serde_yaml::from_reader(std::fs::File::open(path)?)?)
    }
}

impl Walk {
    /// Creates a new instance of [`Walk`] with the specified base folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided folder path is invalid.
    pub fn new(folder: &Path) -> io::Result<Self> {
        Ok(Self {
            folder: folder.canonicalize()?,
            config: Config::try_from_default_file(folder),
        })
    }

    /// Creates a [`Walk`] instance from the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided folder path is invalid.
    pub fn from_config(folder: &Path, config: &Config) -> io::Result<Self> {
        Ok(Self {
            folder: folder.canonicalize()?,
            config: config.clone(),
        })
    }

    /// Checks if a file should be excluded based on configured exclude
    /// patterns.
    fn should_exclude(&self, path: &Path) -> bool {
        let path = path
            .strip_prefix(&self.folder)
            .unwrap()
            .display()
            .to_string();

        for exclude in &self.config.excludes {
            if exclude.is_match(&path) {
                tracing::trace!("file excluded from configurations");
                return true;
            }
        }
        false
    }

    /// Checks if a file should be included based on configured include
    /// patterns.
    fn should_include(&self, path: &Path) -> bool {
        let path = path
            .strip_prefix(&self.folder)
            .unwrap()
            .display()
            .to_string();

        if self.config.includes.is_empty() {
            return true;
        }

        for include in &self.config.includes {
            if include.is_match(&path) {
                tracing::trace!("file excluded from configurations");
                return true;
            }
        }
        tracing::debug!("file should not be included");
        false
    }

    fn exclude_file_path(&self, path: &Path) -> bool {
        for exclude_file in &self.config.excludes_file_path {
            let absolute_path =
                std::fs::canonicalize(exclude_file).unwrap_or_else(|_| exclude_file.clone());
            if absolute_path == path {
                tracing::debug!("file should not be included");
                return true;
            }
        }

        false
    }

    /// Collects files in the specified folder, respecting exclude and include
    /// patterns.
    #[must_use]
    pub fn get_files(&self) -> Vec<PathBuf> {
        let (tx, rx) = mpsc::channel();
        WalkBuilder::new(&self.folder)
            .build_parallel()
            .run(move || {
                let tx = tx.clone();
                Box::new(move |result| {
                    result.map_or_else(
                        |err| {
                            tracing::error!(err = %err,"dir entry error ");
                        },
                        |entry| {
                            if entry.path().is_file() {
                                let path = entry.path().to_owned();
                                if !self.exclude_file_path(path.as_path()) && !self.should_exclude(path.as_path()) && self.should_include(path.as_path()){
                                    if let Err(err) = tx.send(path.clone()) {
                                        tracing::error!(err = %err,path = %path.display(),"error sending path to tx ");
                                    }
                                }
                            }
                        },
                    );
                    ignore::WalkState::Continue
                })
            });

        rx.into_iter().collect::<Vec<_>>()
    }
}
