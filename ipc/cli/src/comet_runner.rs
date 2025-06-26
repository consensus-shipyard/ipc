use std::ffi::OsStr;
use std::{
    env, fs, io,
    path::PathBuf,
    process::{Command, ExitStatus},
};

static COMET_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/comet"));

/// Writes the embedded `COMET_BIN` binary to a temp file, ensures it's executable,
/// and returns its path. Subsequent calls do nothing if already initialized.
fn init_comet() -> io::Result<PathBuf> {
    let file_name = if cfg!(windows) {
        "cometbft.exe"
    } else {
        "cometbft"
    };

    let mut tmp = env::temp_dir();
    tmp.push(file_name);

    if tmp.exists() {
        return Ok(tmp);
    }

    fs::write(&tmp, COMET_BIN)?;

    // On Unix, set the exec bit; no-op on Windows
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tmp, perms)?;
    }

    Ok(tmp)
}

/// Spawns the embedded Comet BFT binary with the provided arguments.
/// Automatically handles writing and permission setup.
pub fn run_comet<I, S>(args: I) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let path = init_comet()?;
    Command::new(path).args(args).status()
}
