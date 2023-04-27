// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct AbciSettings {
    pub host: String,
    pub port: u32,
    /// Queue size for each ABCI component.
    pub bound: usize,
}

impl AbciSettings {
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Deserialize)]
pub struct DbSettings {
    /// Length of the app state history to keep in the database before pruning; 0 means unlimited.
    ///
    /// This affects how long we can go back in state queries.
    pub state_hist_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    /// Home directory configured on the CLI, to which all paths in settings can be set relative.
    home_dir: PathBuf,
    data_dir: PathBuf,
    builtin_actors_bundle: PathBuf,
    pub abci: AbciSettings,
    pub db: DbSettings,
}

impl Settings {
    /// Load the default configuration from a directory,
    /// then potential overrides specific to the run mode,
    /// then overrides from the local environment.
    pub fn new(config_dir: &Path, home_dir: &Path, run_mode: &str) -> Result<Self, ConfigError> {
        let c = Config::builder()
            .add_source(File::from(config_dir.join("default")))
            // Optional mode specific overrides, checked into git.
            .add_source(File::from(config_dir.join(run_mode)).required(false))
            // Optional local overrides, not checked into git.
            .add_source(File::from(config_dir.join("local")).required(false))
            // Add in settings from the environment (with a prefix of FM)
            // e.g. `FM_DB__DATA_DIR=./foo/bar ./target/app` would set the database location.
            .add_source(
                Environment::with_prefix("fm")
                    .prefix_separator("_")
                    .separator("__"),
            )
            // Set the home directory based on what was passed to the CLI,
            // so everything in the config can be relative to it.
            // The `home_dir` key is not added to `default.toml` so there is no confusion
            // about where it will be coming from.
            .set_override("home_dir", home_dir.to_string_lossy().as_ref())?
            .build()?;

        // Deserialize (and thus freeze) the entire configuration.
        c.try_deserialize()
    }

    /// Make the path relative to `--home-dir`, unless it's an absolute path, then expand any `~` in the beginning.
    fn expand_path(&self, path: &PathBuf) -> PathBuf {
        if path.starts_with("/") {
            return path.clone();
        }
        if path.starts_with("~") {
            return expand_tilde(path);
        }
        expand_tilde(self.home_dir.join(path))
    }

    pub fn data_dir(&self) -> PathBuf {
        self.expand_path(&self.data_dir)
    }

    pub fn builtin_actors_bundle(&self) -> PathBuf {
        self.expand_path(&self.builtin_actors_bundle)
    }
}

/// Expand paths that begin with "~" to `$HOME`.
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref().to_path_buf();
    if !p.starts_with("~") {
        return p;
    }
    if p == Path::new("~") {
        return dirs::home_dir().unwrap_or(p);
    }
    dirs::home_dir()
        .map(|mut h| {
            if h == Path::new("/") {
                // `~/foo` becomes just `/foo` instead of `//foo` if `/` is home.
                p.strip_prefix("~").unwrap().to_path_buf()
            } else {
                h.push(p.strip_prefix("~/").unwrap());
                h
            }
        })
        .unwrap_or(p)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::expand_tilde;
    use super::Settings;

    #[test]
    fn parse_default() {
        let current_dir = PathBuf::from(".");
        let default_dir = PathBuf::from("config");
        Settings::new(&default_dir, &current_dir, "test").unwrap();
    }

    #[test]
    fn tilde_expands_to_home() {
        let home = std::env::var("HOME").expect("should work on Linux");
        let home_project = PathBuf::from(format!("{}/.project", home));
        assert_eq!(expand_tilde("~/.project"), home_project);
        assert_eq!(expand_tilde("/foo/bar"), PathBuf::from("/foo/bar"));
        assert_eq!(expand_tilde("~foo/bar"), PathBuf::from("~foo/bar"));
    }
}
