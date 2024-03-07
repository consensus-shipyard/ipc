// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use anyhow::Context;

use crate::config::Config;

pub struct Fendermint {
    config: Config,
}

impl Fendermint {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn run(&self) -> anyhow::Result<bool> {
        // start the fendermint process
        let mut fendermint = Command::new(self.config.root_dir.join("current/bin/fendermint"))
            .args(self.config.fendermint_params.split_whitespace())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to start fendermint")?;

        let stdout = fendermint.stdout.take().expect("no stdout");
        let stderr = fendermint.stderr.take().expect("no stderr");
        let j1 = thread::spawn(move || {
            for line in BufReader::new(stderr).lines() {
                println!("{}", line.unwrap());
            }
        });
        let j2 = thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                println!("{}", line.unwrap());
            }
        });

        // start a thread to wait for the process to exit
        let please_stop = Arc::new(AtomicBool::new(false));
        let handle = thread::spawn({
            let should_i_stop = please_stop.clone();
            move || {
                // loop until the fendermint process stop
                while !should_i_stop.load(Ordering::SeqCst) {
                    println!("from the spawned thread!");
                    thread::sleep(std::time::Duration::from_millis(1000));
                }
                println!("spawned thread exiting!");
            }
        });

        // wait for process to exit or on signal
        fendermint.wait().context("fendermint failed")?;
        j1.join().unwrap();
        j2.join().unwrap();

        // signal the spawned thread to stop
        please_stop.store(true, Ordering::SeqCst);
        handle.join().unwrap();

        println!("fendermint exited");

        Ok(true)
    }
}
