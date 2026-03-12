// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Verified storage layout constants for the Gateway contract.
//!
//! These are derived from the Solidity compiler `storageLayout` for `GatewayDiamond`.
//! In this repo the reliable source of `storageLayout` is the Hardhat build-info artifacts
//! under `contracts/artifacts/build-info/*.json` (not `contracts/out/*`).
//!
//! Keeping them in one place avoids "magic numbers" duplicated across assembler and checks.

/// `GatewayActorStorage.subnets` mapping slot.
///
/// Derived from `GatewayActorStorage` layout: `subnets` is at slot 22.
pub const SUBNETS_MAPPING_SLOT: u64 = 22;

/// `Subnet.topDownNonce` relative slot inside the `Subnet` struct.
pub const SUBNET_TOPDOWN_NONCE_OFFSET: u64 = 3;

/// Absolute storage slot for `GatewayActorStorage.validatorsTracker.changes.nextConfigurationNumber`.
///
/// Derived from the compiled storage layout:
/// - `GatewayActorStorage.validatorsTracker` starts at slot 11
/// - `ParentValidatorsTracker.changes` is at slot 9
/// - `PowerChangeLog.nextConfigurationNumber` is at slot 0
///   => 11 + 9 + 0 = 20
pub const NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT: u64 = 20;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use serde_json::Value;

    fn load_gateway_diamond_storage_layout() -> Result<Value> {
        // Prefer Hardhat build-info artifacts, which include Solidity `storageLayout`:
        // `contracts/artifacts/build-info/*.json`
        //
        // We resolve paths relative to this crate so tests work regardless of cwd.
        let build_info_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../contracts/artifacts/build-info");

        let entries = std::fs::read_dir(&build_info_dir).with_context(|| {
            format!(
                "failed to read Hardhat build-info directory: {:?}",
                build_info_dir
            )
        })?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let bytes =
                std::fs::read(&path).with_context(|| format!("failed to read {:?}", path))?;
            let json: Value = match serde_json::from_slice(bytes.as_slice()) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let layout = &json["output"]["contracts"]["contracts/GatewayDiamond.sol"]
                ["GatewayDiamond"]["storageLayout"];
            if layout["storage"].as_array().is_some() && layout["types"].as_object().is_some() {
                return Ok(layout.clone());
            }
        }

        anyhow::bail!(
            "no Hardhat build-info artifact contained GatewayDiamond storageLayout (expected under contracts/artifacts/build-info)"
        )
    }

    fn parse_slot_u64(v: &Value) -> Result<u64> {
        // Foundry encodes slot as a decimal string (e.g. "22").
        let s = v.as_str().context("expected storage slot to be a string")?;
        s.parse::<u64>()
            .with_context(|| format!("invalid slot string {s:?}"))
    }

    #[test]
    fn storage_layout_constants_match_gateway_diamond_artifact() -> Result<()> {
        let layout = load_gateway_diamond_storage_layout()?;

        let storage = layout["storage"]
            .as_array()
            .context("storageLayout.storage must be an array")?;
        let types = layout["types"]
            .as_object()
            .context("storageLayout.types must be an object")?;

        // Helper: find a member by label in a struct type.
        fn struct_member<'a>(
            types: &'a serde_json::Map<String, Value>,
            struct_type: &str,
            member_label: &str,
        ) -> Result<(&'a Value, u64, &'a str)> {
            let def = types
                .get(struct_type)
                .with_context(|| format!("missing type {struct_type} in storageLayout.types"))?;
            let members = def["members"]
                .as_array()
                .context("type missing members array")?;
            let m = members
                .iter()
                .find(|m| m["label"].as_str() == Some(member_label))
                .with_context(|| format!("missing member {member_label} in type {struct_type}"))?;
            let slot = parse_slot_u64(&m["slot"])?;
            let ty = m["type"].as_str().context("member.type must be a string")?;
            Ok((def, slot, ty))
        }

        // Foundry `GatewayDiamond` artifacts may store the whole `GatewayActorStorage` under a single
        // top-level storage variable (often `s`/`store`), so `subnets`/`validatorsTracker` might not
        // appear as top-level labels. We locate the top-level struct which contains `subnets`.
        let (gateway_base_slot, gateway_struct_type) = storage
            .iter()
            .filter_map(|e| {
                let slot = parse_slot_u64(&e["slot"]).ok()?;
                let ty = e["type"].as_str()?;
                let def = types.get(ty)?;
                def.get("members")?.as_array()?;
                Some((slot, ty.to_string()))
            })
            .find(|(_, ty)| {
                types
                    .get(ty)
                    .and_then(|def| def.get("members"))
                    .and_then(|m| m.as_array())
                    .map(|members| {
                        members
                            .iter()
                            .any(|m| m["label"].as_str() == Some("subnets"))
                    })
                    .unwrap_or(false)
            })
            .context("could not find GatewayActorStorage-like struct containing member=subnets")?;

        // 1) `GatewayActorStorage.subnets` mapping absolute slot.
        let (_gateway_def, subnets_rel_slot, subnets_mapping_type) =
            struct_member(types, &gateway_struct_type, "subnets")?;
        let subnets_abs_slot = gateway_base_slot + subnets_rel_slot;
        assert_eq!(subnets_abs_slot, SUBNETS_MAPPING_SLOT);

        // 2) `Subnet.topDownNonce` relative slot inside the `Subnet` struct (mapping value type).
        let mapping_def = types
            .get(subnets_mapping_type)
            .context("missing subnets mapping type")?;
        let subnet_value_type = mapping_def["value"]
            .as_str()
            .context("subnets mapping type missing .value")?;
        let subnet_def = types
            .get(subnet_value_type)
            .context("missing Subnet struct type")?;
        let subnet_members = subnet_def["members"]
            .as_array()
            .context("Subnet type missing members array")?;
        let topdown_nonce_member = subnet_members
            .iter()
            .find(|m| m["label"].as_str() == Some("topDownNonce"))
            .context("Subnet members missing topDownNonce")?;
        let topdown_nonce_slot = parse_slot_u64(&topdown_nonce_member["slot"])?;
        assert_eq!(topdown_nonce_slot, SUBNET_TOPDOWN_NONCE_OFFSET);

        // 3) Absolute slot for `validatorsTracker.changes.nextConfigurationNumber`.
        let (_gateway_def, vt_rel_slot, vt_type) =
            struct_member(types, &gateway_struct_type, "validatorsTracker")?;
        let (_vt_def, changes_rel_slot, changes_type) = struct_member(types, vt_type, "changes")?;
        let (_changes_def, next_cfg_rel_slot, _next_cfg_type) =
            struct_member(types, changes_type, "nextConfigurationNumber")?;

        let derived_abs = gateway_base_slot + vt_rel_slot + changes_rel_slot + next_cfg_rel_slot;
        assert_eq!(derived_abs, NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT);

        Ok(())
    }
}
