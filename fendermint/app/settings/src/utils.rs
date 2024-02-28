// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! home_relative {
    // Using this inside something that has a `.home_dir()` function.
    ($($name:ident),+) => {
        $(
        pub fn $name(&self) -> std::path::PathBuf {
            crate::utils::expand_path(&self.home_dir(), &self.$name)
        }
        )+
    };

    // Using this outside something that requires a `home_dir` parameter to be passed to it.
    ($settings:ty { $($name:ident),+ } ) => {
      impl $settings {
        $(
        pub fn $name(&self, home_dir: &std::path::Path) -> std::path::PathBuf {
            $crate::utils::expand_path(home_dir, &self.$name)
        }
        )+
      }
    };
}

/// Expand a path which can either be :
/// * absolute, e.g. "/foo/bar"
/// * relative to the system `$HOME` directory, e.g. "~/foo/bar"
/// * relative to the configured `--home-dir` directory, e.g. "foo/bar"
pub fn expand_path(home_dir: &Path, path: &Path) -> PathBuf {
    if path.starts_with("/") {
        PathBuf::from(path)
    } else if path.starts_with("~") {
        expand_tilde(path)
    } else {
        expand_tilde(home_dir.join(path))
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
    use super::expand_tilde;

    #[test]
    fn tilde_expands_to_home() {
        let home = std::env::var("HOME").expect("should work on Linux");
        let home_project = PathBuf::from(format!("{}/.project", home));
        assert_eq!(expand_tilde("~/.project"), home_project);
        assert_eq!(expand_tilde("/foo/bar"), PathBuf::from("/foo/bar"));
        assert_eq!(expand_tilde("~foo/bar"), PathBuf::from("~foo/bar"));
    }
}
