// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use fs_err as fs;
use std::path::Path;
use toml::Value as TomlValue;

/// Merge TOML overrides into existing TOML config file
pub fn merge_toml_config(config_path: &Path, overrides: &TomlValue) -> anyhow::Result<()> {
    let existing_content = fs::read_to_string(config_path)?;
    let mut existing_config: TomlValue = toml::from_str(&existing_content)?;

    deep_merge(&mut existing_config, overrides);

    let new_content = toml::to_string_pretty(&existing_config)?;
    fs::write(config_path, new_content)?;

    log::info!(
        "Applied configuration overrides to: {}",
        config_path.display()
    );
    Ok(())
}

/// Deep merge two TOML values
fn deep_merge(target: &mut TomlValue, source: &TomlValue) {
    match (target, source) {
        (TomlValue::Table(target_table), TomlValue::Table(source_table)) => {
            for (key, value) in source_table {
                if let Some(target_value) = target_table.get_mut(key) {
                    deep_merge(target_value, value);
                } else {
                    target_table.insert(key.clone(), value.clone());
                }
            }
        }
        (target, source) => {
            *target = source.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_deep_merge_simple_values() {
        let mut target = toml::from_str::<TomlValue>("a = 1").unwrap();
        let source = toml::from_str::<TomlValue>("a = 2").unwrap();

        deep_merge(&mut target, &source);

        assert_eq!(target.get("a").unwrap().as_integer().unwrap(), 2);
    }

    #[test]
    fn test_deep_merge_nested_tables() {
        let mut target = toml::from_str::<TomlValue>(
            r#"
            [consensus]
            timeout_commit = "1s"
            [rpc]
            laddr = "tcp://127.0.0.1:26657"
        "#,
        )
        .unwrap();

        let source = toml::from_str::<TomlValue>(
            r#"
            [consensus]
            timeout_commit = "5s"
            [p2p]
            laddr = "tcp://0.0.0.0:26656"
        "#,
        )
        .unwrap();

        deep_merge(&mut target, &source);

        // Check that existing values are updated
        assert_eq!(
            target
                .get("consensus")
                .unwrap()
                .get("timeout_commit")
                .unwrap()
                .as_str()
                .unwrap(),
            "5s"
        );

        // Check that existing values not in source are preserved
        assert_eq!(
            target
                .get("rpc")
                .unwrap()
                .get("laddr")
                .unwrap()
                .as_str()
                .unwrap(),
            "tcp://127.0.0.1:26657"
        );

        // Check that new values are added
        assert_eq!(
            target
                .get("p2p")
                .unwrap()
                .get("laddr")
                .unwrap()
                .as_str()
                .unwrap(),
            "tcp://0.0.0.0:26656"
        );
    }

    #[test]
    fn test_deep_merge_empty_source() {
        let mut target = toml::from_str::<TomlValue>(
            r#"
            [consensus]
            timeout_commit = "1s"
        "#,
        )
        .unwrap();

        let source = toml::from_str::<TomlValue>("").unwrap();

        deep_merge(&mut target, &source);

        // Target should remain unchanged
        assert_eq!(
            target
                .get("consensus")
                .unwrap()
                .get("timeout_commit")
                .unwrap()
                .as_str()
                .unwrap(),
            "1s"
        );
    }

    #[test]
    fn test_merge_toml_config_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create initial config file
        let initial_config = r#"
            [consensus]
            timeout_commit = "1s"
            [rpc]
            laddr = "tcp://127.0.0.1:26657"
        "#;
        fs::write(&config_path, initial_config).unwrap();

        // Define overrides
        let overrides = toml::from_str::<TomlValue>(
            r#"
            [consensus]
            timeout_commit = "5s"
            [p2p]
            laddr = "tcp://0.0.0.0:26656"
        "#,
        )
        .unwrap();

        // Apply overrides
        merge_toml_config(&config_path, &overrides).unwrap();

        // Read back and verify
        let content = fs::read_to_string(&config_path).unwrap();
        let merged_config: TomlValue = toml::from_str(&content).unwrap();

        assert_eq!(
            merged_config
                .get("consensus")
                .unwrap()
                .get("timeout_commit")
                .unwrap()
                .as_str()
                .unwrap(),
            "5s"
        );
        assert_eq!(
            merged_config
                .get("rpc")
                .unwrap()
                .get("laddr")
                .unwrap()
                .as_str()
                .unwrap(),
            "tcp://127.0.0.1:26657"
        );
        assert_eq!(
            merged_config
                .get("p2p")
                .unwrap()
                .get("laddr")
                .unwrap()
                .as_str()
                .unwrap(),
            "tcp://0.0.0.0:26656"
        );
    }

    #[test]
    fn test_merge_toml_config_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");

        let overrides = toml::from_str::<TomlValue>("a = 1").unwrap();

        // Should fail because file doesn't exist
        let result = merge_toml_config(&config_path, &overrides);
        assert!(result.is_err());
    }
}
