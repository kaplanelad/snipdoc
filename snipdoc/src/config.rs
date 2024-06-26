//! This module provides configuration management for the `snipdoc`. It
//! includes functionality to load and manage configurations from a default YAML
//! file.
use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::errors::ConfigResult;

pub const DEFAULT_CONFIG_NAME: &str = "snipdoc-config.yml";

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub walk: WalkConfig,
    #[serde(default)]
    pub inject: InjectConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct InjectConfig {}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct WalkConfig {
    /// Patterns to include files.
    #[serde(with = "serde_regex", default)]
    pub includes: Vec<Regex>,
    /// Patterns to exclude files.
    #[serde(with = "serde_regex", default)]
    pub excludes: Vec<Regex>,
}

impl Config {
    /// Attempts to load the configuration from a default file under the given
    /// path.
    ///
    /// This function first checks if the default configuration file exists at
    /// the given path. If it exists, it tries to load the configuration
    /// from this file. If the file does not exist or contains invalid
    /// content, it falls back to loading the default configuration.
    ///
    /// # Returns
    ///
    /// A [`Config`] instance loaded from the file if it exists and is valid, or
    /// a default `Config` instance otherwise.
    pub fn try_from_default_file(path: &Path) -> Self {
        let maybe_config_exists = path.join(DEFAULT_CONFIG_NAME);

        if maybe_config_exists.exists() {
            match Self::from_file(maybe_config_exists.as_path()) {
                Ok(config) => {
                    tracing::debug!(
                        path = %maybe_config_exists.display(),
                        "config file loaded"
                    );
                    return config;
                }
                Err(err) => {
                    tracing::error!(
                        path = %maybe_config_exists.display(),
                        err = %err,
                        "invalid config file content"
                    );
                }
            }
        } else {
            tracing::debug!(
                path = %maybe_config_exists.display(),
                "config not exists"
            );
        }
        Self::default()
    }

    /// Loads the configuration from the specified file.
    ///
    /// This function reads the file at the given path and attempts to
    /// deserialize its content into a [`Config`] instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the content is
    /// invalid.
    pub fn from_file(path: &Path) -> ConfigResult<'_, Self> {
        Ok(serde_yaml::from_reader(std::fs::File::open(path)?)?)
    }
}
