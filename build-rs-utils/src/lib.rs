// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Utilites for usage in `build.rs` files.

use std::io::BufReader;
use std::process::Stdio;
use std::sync::mpsc;

use color_eyre::eyre::Result;
pub use owo_colors::*;

pub fn rerun_if_changed(path: impl AsRef<std::path::Path>) {
    ::std::println!("cargo:rerun-if-changed={}", path.as_ref().display());
}

pub fn rerun_if_env_changed(env_var_name: impl std::fmt::Display) {
    ::std::println!("cargo:rerun-if-env-changed={}", env_var_name);
}

#[macro_export]
macro_rules! build_print {
    ($topic:expr) => {
        ::std::println!("cargo:warning=\x1b[2K\r{}", $topic.bold())
    };
    ($topic:expr, $($arg:tt)+) => {
        ::std::println!("cargo:warning=\x1b[2K\r   {}: {}", $topic.bold(), ::std::format!($($arg)*))
    };
}

/// Echo some text with a prefix
///
/// ```no_run
/// # use crate::echo;
/// let red = "some redish color tone";
/// echo!("foo", cyan, "I like {red}");
/// ```
///
/// resulting in
///
/// ```html
///     <style color="cyan" font-weight="bold">foo</style>: I like some redish color tone
/// ```
#[macro_export]
macro_rules! echo {
    ($component:expr, cyan, $($arg:tt)* ) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.cyan(), $($arg)*)
        }
    };
    ($component:expr, blue, $($arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.blue(), $($arg)*)
        }
    };
    ($component:expr, orange, $($arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.orange(), $($arg)*)
        }
    };
    ($component:expr, purple, $($arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.purple(), $($arg)*)
        }
    };
    ($component:expr, red, $( $arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.red(), $($arg)*)
        }
    };
    ($component:expr, green, $($arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.green(), $($arg)*)
        }
    };
    ($component:expr, yellow, $($arg:tt)*) => {
        {
            use $crate::OwoColorize as _;
            $crate::build_print!($component.yellow(), $($arg)*)
        }
    };
}

/// Launch a command with attached stdio, piping them into the current process using the `name` as prefix.
///
/// It assumes output are printable strings and valid UTF-8
pub async fn run_command_with_stdio(
    name: &'static str,
    mut cmd: std::process::Command,
) -> Result<()> {
    // handle concurrent output lines
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    #[derive(Debug)]
    enum What {
        Stderr(String),
        Stdout(String),
        Exit(std::io::Result<()>),
    }

    let (tx, rx) = mpsc::channel();
    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let tx = tx.clone();

    fn read_on(
        tx: mpsc::Sender<What>,
        mut reader: impl std::io::BufRead,
        what: impl Fn(String) -> What,
    ) {
        let mut buf = String::with_capacity(1024);
        loop {
            match reader.read_line(&mut buf) {
                Ok(0) => {
                    let _ = tx.send(What::Exit(Ok(())));
                    break;
                }
                Ok(_n) => {
                    for line in buf.lines() {
                        let _ = tx.send(what(line.trim().to_string()));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                e => {
                    let _ = tx.send(What::Exit(e.map(|_| ())));
                    break;
                }
            }
        }
    }
    let tx_cc = tx.clone();
    let jh1 =
        tokio::task::spawn_blocking(move || read_on(tx_cc, BufReader::new(stderr), What::Stderr));
    let jh2 =
        tokio::task::spawn_blocking(move || read_on(tx, BufReader::new(stdout), What::Stdout));

    // deduplication - some tools print the identical text/line multiple times
    let mut previous_stdout_line = None;
    let mut previous_stderr_line = None;

    fn deduplicate(msg: String, previous: &mut Option<String>) -> Option<String> {
        let vis = msg
            .as_str()
            .rsplit("\x1b[2K\r")
            .next()
            .unwrap_or_else(|| msg.as_str());
        if previous.as_ref().map(|x| x.as_str()) == Some(vis) {
            return None;
        }
        let vis = vis.to_string();
        previous.replace(vis.clone());
        Some(vis)
    }

    while let Ok(x) = rx.recv() {
        match x {
            What::Stderr(msg) => {
                let Some(msg) = deduplicate(msg, &mut previous_stderr_line) else {
                    continue;
                };
                echo!(name, cyan, "(err) {}", msg);
            }
            What::Stdout(msg) => {
                let Some(msg) = deduplicate(msg, &mut previous_stdout_line) else {
                    continue;
                };
                echo!(name, purple, "(out) {}", msg);
            }
            What::Exit(res) => {
                res?;
                break;
            }
        }
    }
    jh1.await?;
    jh2.await?;

    Ok(())
}
