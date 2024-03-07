// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use std::error::Error;

use clap::{Parser, Subcommand};
use ipcvisor::{
    init::{self, InitArgs},
    run::{self, RunArgs},
};

#[derive(Debug, Parser)] // requires `derive` feature
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Init(InitArgs),
    Run(RunArgs),
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    match args.command {
        Commands::Init(args) => init::init(args)?,
        Commands::Run(args) => run::run(args)?,
    }

    Ok(())
}
