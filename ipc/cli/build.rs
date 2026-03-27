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
const VERSION: &str = "v0.38.17";

// SHA-256 sums for v0.38.17 assets
const CHECKSUMS: &[(&str, &str)] = &[
    (
        "cometbft_0.38.17_darwin_amd64.tar.gz",
        "5a69c2c0f26a65a3b3b17ba62da0615a552ea897449e63d0d5ec62997b901eae",
    ),
    (
        "cometbft_0.38.17_darwin_arm64.tar.gz",
        "0809221df1ed6b1471b266a8bbdc673331c7ac45f26c20af91bb383e3e6457a5",
    ),
    (
        "cometbft_0.38.17_linux_amd64.tar.gz",
        "ca4d7ca0df296e092462edc92a3f07a4cd1e6c0366516e382a0726b522806f38",
    ),
    (
        "cometbft_0.38.17_linux_arm64.tar.gz",
        "5a6a4bf655a84d9463932ff93ac373ea68327c14c26d65a497a3907c45c65144",
    ),
    (
        "cometbft_0.38.17_linux_armv6.tar.gz",
        "95fae10f5fc5c0ff8176d9497f684c2a0a27c15fc0f96bb3e131489db14d0c1a",
    ),
    (
        "cometbft_0.38.17_windows_amd64.tar.gz",
        "3523d602f355ef73380d91db335f86daa4d6e0848f54a09cdbc918defb38281c",
    ),
    (
        "cometbft_0.38.17_windows_arm64.tar.gz",
        "1bfe9a6e2bceccee679a9917363bbb3b167c1387be821f3f4c661d226a38b749",
    ),
    (
        "cometbft_0.38.17_windows_armv6.tar.gz",
        "cf70cdcabd82e9b7080054d06665d286a026469162715057b7ab458cf693fcb0",
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
        ("macos", "x86_64") => "cometbft_0.38.17_darwin_amd64.tar.gz",
        ("macos", "aarch64") => "cometbft_0.38.17_darwin_arm64.tar.gz",
        ("linux", "x86_64") => "cometbft_0.38.17_linux_amd64.tar.gz",
        ("linux", "aarch64") => "cometbft_0.38.17_linux_arm64.tar.gz",
        ("linux", "arm") => "cometbft_0.38.17_linux_armv6.tar.gz",
        ("windows", "x86_64") => "cometbft_0.38.17_windows_amd64.tar.gz",
        ("windows", "aarch64") => "cometbft_0.38.17_windows_arm64.tar.gz",
        ("windows", "arm") => "cometbft_0.38.17_windows_armv6.tar.gz",
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
