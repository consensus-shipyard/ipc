use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{error, info};

/// A process manager that holds the configuration for a process
/// and provides methods to spawn and execute it.
pub struct ProcessManager<'a> {
    pub binary: PathBuf,
    pub args: Vec<&'a str>,
}

impl<'a> ProcessManager<'a> {
    pub fn new(binary: PathBuf, args: Vec<&'a str>) -> Self {
        Self { binary, args }
    }

    /// Spawns the process with piped stdout and stderr and attaches logging.
    pub async fn spawn(&self) -> Result<Child> {
        let mut child = Command::new(&self.binary)
            .args(&self.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to spawn process: {:?}", self.binary))?;

        // Setup logging for stdout.
        if let Some(stdout) = child.stdout.take() {
            let mut stdout_reader = BufReader::new(stdout).lines();
            let bin = self.binary.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stdout_reader.next_line().await {
                    info!(target: "process", binary = ?bin, "stdout: {}", line);
                }
            });
        } else {
            error!(target: "process", binary = ?self.binary, "Child process had no stdout");
        }

        // Setup logging for stderr.
        if let Some(stderr) = child.stderr.take() {
            let mut stderr_reader = BufReader::new(stderr).lines();
            let bin = self.binary.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_reader.next_line().await {
                    error!(target: "process", binary = ?bin, "stderr: {}", line);
                }
            });
        } else {
            error!(target: "process", binary = ?self.binary, "Child process had no stderr");
        }

        Ok(child)
    }

    /// Executes the process one-off, waiting for it to finish.
    /// For a long-running process, this will await until termination.
    pub async fn execute(&self) -> Result<()> {
        let mut child = self.spawn().await?;
        info!(target: "process", binary = ?self.binary, "Process started");

        let status = child
            .wait()
            .await
            .with_context(|| format!("Failed while waiting for process: {:?}", self.binary))?;
        info!(target: "process", binary = ?self.binary, "Process exited with status: {:?}", status);

        Ok(())
    }
}

/// Gracefully kills a process, if it is running.
pub async fn kill_process(child: &mut Child, name: &str) {
    if let Some(pid) = child.id() {
        let _ = child.kill().await;
        info!("{} (pid={}) has been killed.", name, pid);
    }
}
