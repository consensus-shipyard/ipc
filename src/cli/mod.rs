use async_trait::async_trait;
use clap::Args;

mod commands;

pub use commands::cli;

/// The trait that represents the abstraction of a command line handler. To implement a new command
/// line operation, implement this trait and register it.
///
/// Note that this trait does not support a stateful implementation as we assume CLI commands are all
/// constructed from scratch.
#[async_trait]
pub trait CommandLineHandler {
    /// Abstraction for command line operations arguments.
    ///
    /// NOTE that this parameter is used to generate the command line arguments.
    /// Currently we are directly integrating with `clap` crate. In the future we can use our own
    /// implementation to abstract away external crates. But this should be good for now.
    type Arguments: std::fmt::Debug + Args;

    /// Handles the request with the provided arguments. Dev should handle the content to print and how
    async fn handle(arguments: &Self::Arguments) -> anyhow::Result<()>;
}
