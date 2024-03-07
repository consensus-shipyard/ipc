use std::{os::unix::fs::PermissionsExt, path::Path};

use anyhow::Context;
use serde::{de::DeserializeOwned, Serialize};

/// Export text to a file.
pub fn export_file(file_path: impl AsRef<Path>, contents: impl AsRef<str>) -> anyhow::Result<()> {
    if let Some(dir_path) = file_path.as_ref().parent() {
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path).with_context(|| {
                format!("failed to create directory {}", dir_path.to_string_lossy())
            })?;
        }
    }

    std::fs::write(&file_path, contents.as_ref()).with_context(|| {
        format!(
            "failed to write to {}",
            file_path.as_ref().to_string_lossy()
        )
    })?;

    Ok(())
}

/// Export executable shell script.
pub fn export_script(file_path: impl AsRef<Path>, contents: impl AsRef<str>) -> anyhow::Result<()> {
    export_file(&file_path, contents)?;

    std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o774))
        .context("failed to set file permissions")?;

    Ok(())
}

/// Export an object as JSON.
pub fn export_json(file_path: impl AsRef<Path>, value: impl Serialize) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(&value).context("failed to serialize to JSON")?;

    export_file(file_path, json)
}

/// Read a JSON file, if it exists.
pub fn import_json<T: DeserializeOwned>(file_path: impl AsRef<Path>) -> anyhow::Result<Option<T>> {
    let file_path = file_path.as_ref();
    if file_path.exists() {
        let json = std::fs::read_to_string(file_path)
            .with_context(|| format!("failed to read {}", file_path.to_string_lossy()))?;

        let value = serde_json::from_str::<T>(&json)
            .with_context(|| format!("failed to parse {}", file_path.to_string_lossy()))?;

        Ok(Some(value))
    } else {
        Ok(None)
    }
}
