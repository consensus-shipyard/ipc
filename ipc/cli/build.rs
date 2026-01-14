// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

// build.rs

use flate2::read::GzDecoder;
use hex::decode as hex_decode;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, ACCEPT, USER_AGENT};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{self},
    path::PathBuf,
    process::exit,
};
use tar::Archive;

#[derive(Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    id: u64,
    name: String,
}

const OWNER: &str = "cometbft";
const REPO: &str = "cometbft";
const VERSION: &str = "v0.37.15";

// SHA-256 sums for v0.37.15 assets
const CHECKSUMS: &[(&str, &str)] = &[
    (
        "cometbft_0.37.15_darwin_amd64.tar.gz",
        "693958a935325eb21269ccf89c39e9feb88e5be663cfb9510deae5e41af5acb7",
    ),
    (
        "cometbft_0.37.15_darwin_arm64.tar.gz",
        "933b97c4ea40eba0d41f71654a276774535cbc54400e4b361d73545246d60b23",
    ),
    (
        "cometbft_0.37.15_linux_amd64.tar.gz",
        "617846c4d146323564d2c2759592d30b9357b3c8e126e9e066ac95d16e146d4d",
    ),
    (
        "cometbft_0.37.15_linux_arm64.tar.gz",
        "b25aab414c64183aa401752cba4b03609cf035f5b587376eeac840895c1e36c1",
    ),
    (
        "cometbft_0.37.15_linux_armv6.tar.gz",
        "63b98b93a7708dc6a0b1798b8b186d575a2787da158d67bc787aac3de00e0da8",
    ),
    (
        "cometbft_0.37.15_windows_amd64.tar.gz",
        "e42cf4e1d84b056630248dcaecb1bddea43b668e121ee4ea19587494a3367187",
    ),
    (
        "cometbft_0.37.15_windows_arm64.tar.gz",
        "880fb704519b7005a5f395f10b0dcc4fbc7dd4936acacd8b1a373ae76e775572",
    ),
    (
        "cometbft_0.37.15_windows_armv6.tar.gz",
        "5041df72a5ec2ceb3d0f3dbec7272c507c2f92e0d22cfb1e04b26370e32f6de4",
    ),
];

fn main() {
    // Build headers for GitHub API
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        format!("build.rs/{}/{}", OWNER, REPO).parse().unwrap(),
    );

    let client = Client::builder()
        .default_headers(headers)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("HTTP client error: {}", e);
            exit(1)
        });

    // 1) Fetch release metadata
    let release_url = format!(
        "https://api.github.com/repos/{}/{}/releases/tags/{}",
        OWNER, REPO, VERSION
    );
    let release: Release = client
        .get(&release_url)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.json())
        .unwrap_or_else(|e| {
            eprintln!("Failed to fetch release: {}", e);
            exit(1)
        });

    // 2) Determine asset filename
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let asset_name = match (os.as_str(), arch.as_str()) {
        ("macos", "x86_64") => "cometbft_0.37.15_darwin_amd64.tar.gz",
        ("macos", "aarch64") => "cometbft_0.37.15_darwin_arm64.tar.gz",
        ("linux", "x86_64") => "cometbft_0.37.15_linux_amd64.tar.gz",
        ("linux", "aarch64") => "cometbft_0.37.15_linux_arm64.tar.gz",
        ("linux", "arm") => "cometbft_0.37.15_linux_armv6.tar.gz",
        ("windows", "x86_64") => "cometbft_0.37.15_windows_amd64.tar.gz",
        ("windows", "aarch64") => "cometbft_0.37.15_windows_arm64.tar.gz",
        ("windows", "arm") => "cometbft_0.37.15_windows_armv6.tar.gz",
        _ => {
            eprintln!("Unsupported target: {}-{}", os, arch);
            exit(1)
        }
    };

    // 3) Find the matching asset
    let asset = release
        .assets
        .into_iter()
        .find(|a| a.name == asset_name)
        .unwrap_or_else(|| {
            eprintln!("Asset {} not found", asset_name);
            exit(1)
        });

    // 4) Download via API
    let asset_url = format!(
        "https://api.github.com/repos/{}/{}/releases/assets/{}",
        OWNER, REPO, asset.id
    );
    let mut resp = client
        .get(&asset_url)
        .header(ACCEPT, "application/octet-stream")
        .send()
        .and_then(|r| r.error_for_status())
        .unwrap_or_else(|e| {
            eprintln!("Download error: {}", e);
            exit(1)
        });
    let mut data = Vec::new();
    resp.copy_to(&mut data).unwrap_or_else(|e| {
        eprintln!("Read body error: {}", e);
        exit(1)
    });

    // 5) Verify checksum
    let expected = CHECKSUMS
        .iter()
        .find_map(|(n, h)| if *n == asset_name { Some(*h) } else { None })
        .expect("checksum missing");
    let mut hasher = Sha256::new();
    hasher.update(&data);
    if *hasher.finalize() != hex_decode(expected).unwrap()[..] {
        eprintln!("Checksum mismatch for {}", asset_name);
        exit(1);
    }

    // 6) Unpack single binary to OUT_DIR/comet
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_bin = out_dir.join("comet");
    let mut archive = Archive::new(GzDecoder::new(&data[..]));
    let mut file = fs::File::create(&out_bin).unwrap();
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let entry_path = entry.path().unwrap().to_path_buf();
        let file_name = entry_path.file_name().unwrap().to_string_lossy();
        if file_name.starts_with("cometbft") {
            io::copy(&mut entry, &mut file).unwrap();
            break;
        }
    }

    // 7) Cargo re-run directive
    println!("cargo:rerun-if-changed=build.rs");
}
