// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod binary;
mod closure;

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub const BINARY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../binaries/cometbft/darwin_amd64"
);

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const BINARY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../binaries/cometbft/darwin_arm64"
);

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const BINARY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../binaries/cometbft/linux_amd64"
);

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const BINARY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../binaries/cometbft/linux_arm64"
);

#[cfg(all(target_os = "linux", target_arch = "arm"))]
pub const BINARY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../binaries/cometbft/linux_armv6"
);

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const BINARY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../binaries/cometbft/windows_amd64.exe"
);

#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
pub const BINARY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../binaries/cometbft/windows_arm64.exe"
);

#[cfg(all(target_os = "windows", target_arch = "arm"))]
pub const BINARY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../binaries/cometbft/windows_armv6.exe"
);

// If none of the above conditions match, trigger a compile-time error.
#[cfg(not(any(
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "arm"),
    all(target_os = "windows", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "arm")
)))]
compile_error!("Unsupported platform or architecture");
