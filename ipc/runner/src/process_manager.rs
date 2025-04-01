use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

/// Configuration for a process to be managed.
pub struct ProcessConfig {
    // TODO Karel - use Path instead
    pub binary: String,
    pub args: Vec<String>,
}

/// Spawns a process based on the given configuration and pipes its output.
pub async fn spawn_process(config: &ProcessConfig) -> anyhow::Result<Child> {
    let mut child = Command::new(&config.binary)
        .args(&config.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Pipe stdout and stderr so that logs are integrated with our tracing.
    let stdout = child.stdout.take().expect("Expected stdout");
    let stderr = child.stderr.take().expect("Expected stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    // Log stdout asynchronously.
    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            tracing::info!(target: "process", "{}", line);
        }
    });
    // Log stderr asynchronously.
    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            tracing::error!(target: "process", "{}", line);
        }
    });

    Ok(child)
}

/// Gracefully kills a process, if it is running.
pub async fn kill_process(child: &mut Child, name: &str) {
    if let Some(pid) = child.id() {
        let _ = child.kill().await;
        tracing::info!("{} (pid={}) has been killed.", name, pid);
    }
}
