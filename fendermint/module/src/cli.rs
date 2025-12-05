// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! CLI module trait for adding custom commands.
//!
//! This trait allows modules to extend the CLI with their own commands
//! and subcommands.

use anyhow::Result;
use async_trait::async_trait;
use std::fmt;

/// A CLI command definition.
///
/// This represents a command or subcommand that can be added to the CLI.
/// Commands can be nested to create complex command hierarchies.
#[derive(Debug, Clone)]
pub struct CommandDef {
    /// The command name (e.g., "objects")
    pub name: String,
    /// A short description of what the command does
    pub about: String,
    /// Optional long description with more details
    pub long_about: Option<String>,
    /// Subcommands nested under this command
    pub subcommands: Vec<CommandDef>,
    /// Whether this command is hidden in help output
    pub hidden: bool,
}

impl CommandDef {
    /// Create a new command definition.
    pub fn new(name: impl Into<String>, about: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            about: about.into(),
            long_about: None,
            subcommands: vec![],
            hidden: false,
        }
    }

    /// Set the long description.
    pub fn long_about(mut self, long_about: impl Into<String>) -> Self {
        self.long_about = Some(long_about.into());
        self
    }

    /// Add a subcommand.
    pub fn subcommand(mut self, cmd: CommandDef) -> Self {
        self.subcommands.push(cmd);
        self
    }

    /// Mark this command as hidden.
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }
}

/// Arguments passed to a command when it's executed.
///
/// This is a simplified representation that modules can use to
/// access command-line arguments.
#[derive(Debug, Clone)]
pub struct CommandArgs {
    /// The command name that was invoked
    pub command: String,
    /// Key-value pairs of arguments
    pub args: Vec<(String, String)>,
    /// Positional arguments
    pub positional: Vec<String>,
}

impl CommandArgs {
    /// Create new command arguments.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: vec![],
            positional: vec![],
        }
    }

    /// Add a named argument.
    pub fn arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.args.push((key.into(), value.into()));
        self
    }

    /// Add a positional argument.
    pub fn positional(mut self, value: impl Into<String>) -> Self {
        self.positional.push(value.into());
        self
    }

    /// Get the value of a named argument.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.args
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Get a positional argument by index.
    pub fn get_positional(&self, index: usize) -> Option<&str> {
        self.positional.get(index).map(|s| s.as_str())
    }
}

/// Module trait for adding custom CLI commands.
///
/// Modules can implement this trait to extend the CLI with additional
/// commands. This is useful for administration tasks, debugging tools,
/// or any other functionality that should be accessible from the command line.
///
/// # Example
///
/// ```ignore
/// struct MyModule;
///
/// #[async_trait]
/// impl CliModule for MyModule {
///     fn commands(&self) -> Vec<CommandDef> {
///         vec![
///             CommandDef::new("mycommand", "Do something useful")
///                 .subcommand(
///                     CommandDef::new("run", "Run the thing")
///                 )
///                 .subcommand(
///                     CommandDef::new("status", "Check status")
///                 ),
///         ]
///     }
///
///     async fn execute(&self, args: &CommandArgs) -> Result<()> {
///         match args.command.as_str() {
///             "run" => self.run(args).await,
///             "status" => self.status(args).await,
///             _ => bail!("Unknown command: {}", args.command),
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait CliModule: Send + Sync {
    /// Get the list of commands this module provides.
    ///
    /// These commands will be added to the main CLI parser.
    ///
    /// # Returns
    ///
    /// A vector of command definitions
    fn commands(&self) -> Vec<CommandDef>;

    /// Execute a command.
    ///
    /// This is called when a user invokes one of this module's commands.
    ///
    /// # Arguments
    ///
    /// * `args` - The parsed command arguments
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the command executed successfully
    /// * `Err(e)` if the command failed
    async fn execute(&self, args: &CommandArgs) -> Result<()>;

    /// Optional: Validate command arguments before execution.
    ///
    /// This is called before `execute`. Modules can use this to validate
    /// that all required arguments are present and valid.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the arguments are valid
    /// * `Err(e)` if validation failed
    fn validate_args(&self, _args: &CommandArgs) -> Result<()> {
        Ok(()) // Default: no validation
    }

    /// Optional: Provide shell completion hints for arguments.
    ///
    /// This can be used to provide intelligent tab completion in shells.
    ///
    /// # Arguments
    ///
    /// * `command` - The command being completed
    /// * `arg` - The argument being completed
    ///
    /// # Returns
    ///
    /// A list of possible completions
    fn complete(&self, _command: &str, _arg: &str) -> Vec<String> {
        vec![] // Default: no completions
    }
}

/// Default no-op CLI module that doesn't add any commands.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpCliModule;

#[async_trait]
impl CliModule for NoOpCliModule {
    fn commands(&self) -> Vec<CommandDef> {
        vec![] // No commands to add
    }

    async fn execute(&self, args: &CommandArgs) -> Result<()> {
        anyhow::bail!("No CLI commands available (command: {})", args.command)
    }

    fn validate_args(&self, _args: &CommandArgs) -> Result<()> {
        Ok(())
    }

    fn complete(&self, _command: &str, _arg: &str) -> Vec<String> {
        vec![]
    }
}

impl fmt::Display for NoOpCliModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NoOpCliModule")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_def_builder() {
        let cmd = CommandDef::new("test", "Test command")
            .long_about("This is a longer description")
            .subcommand(CommandDef::new("sub", "Subcommand"))
            .hidden(true);

        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.about, "Test command");
        assert!(cmd.long_about.is_some());
        assert_eq!(cmd.subcommands.len(), 1);
        assert!(cmd.hidden);
    }

    #[test]
    fn test_command_args_builder() {
        let args = CommandArgs::new("test")
            .arg("key1", "value1")
            .arg("key2", "value2")
            .positional("pos1")
            .positional("pos2");

        assert_eq!(args.command, "test");
        assert_eq!(args.get("key1"), Some("value1"));
        assert_eq!(args.get("key2"), Some("value2"));
        assert_eq!(args.get_positional(0), Some("pos1"));
        assert_eq!(args.get_positional(1), Some("pos2"));
    }

    #[test]
    fn test_no_op_cli_module_commands() {
        let module = NoOpCliModule;
        assert_eq!(module.commands().len(), 0);
    }

    #[tokio::test]
    async fn test_no_op_cli_module_execute() {
        let module = NoOpCliModule;
        let args = CommandArgs::new("test");
        let result = module.execute(&args).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_no_op_cli_module_validate() {
        let module = NoOpCliModule;
        let args = CommandArgs::new("test");
        let result = module.validate_args(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_op_cli_module_complete() {
        let module = NoOpCliModule;
        let completions = module.complete("test", "arg");
        assert_eq!(completions.len(), 0);
    }
}
