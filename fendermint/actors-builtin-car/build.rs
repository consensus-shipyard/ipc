// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Download filecoin's builtin actors car file

use build_rs_utils::echo;
use bytes::buf::Buf;
use color_eyre::eyre::{self, bail, Result, WrapErr};
use fs_err as fs;
use futures_util::stream::StreamExt;
use sha2::Digest;
use std::io::{Read, Write};
use std::path::Path;
use tempfile::NamedTempFile;

const BUILTIN_ACTORS_TAG: &str = "v15.0.0";
const BUILTIN_ACTORS_SHA256SUM: &str =
    "fd7e442bb52ee2e0079053eaf4f75670257f2084b6315826e084c2b483deee4a";

const FORCE_RERUN: &str = "IPC_BUILTIN_ACTORS_FORCE_FETCH";

const VERSION_OVERRIDE: &str = "IPC_BUILTIN_ACTORS_VERSION_OVERRIDE";

/// Handle `Interrupt` errors, call a closure for each read kb piece
fn read_file_piecewise<R: Read, F: FnMut(&[u8]) -> Result<()>>(
    mut reader: R,
    mut f: F,
) -> Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        match reader.read(&mut buf[..]) {
            Ok(0) => {
                return Ok(());
            }
            Ok(n) => {
                f(&buf[..n])?;
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::Interrupted => continue,
                _ => return Err(e.into()),
            },
        }
    }
}

/// Calculate the digest of a file
fn file_digest(reader: impl Read) -> Result<Checksum> {
    let mut sha = sha2::Sha256::default();
    read_file_piecewise(reader, |bytes| {
        sha.write_all(bytes)?;
        Ok(())
    })?;
    let digest = sha.finalize();
    Checksum::try_from(digest.as_slice())
}

/// Convert a slice to an array
///
/// The slice must have the exact length of the array size `N`.
fn slice_to_array<const N: usize>(bytes: &[u8]) -> Result<[u8; N]> {
    if bytes.len() != N {
        bail!("Length mismatch, execpted {}, but got {}", N, bytes.len());
    }
    let mut buf = [0u8; N];
    buf.copy_from_slice(&bytes[..N]);
    Ok(buf)
}

/// Digest wrapping type
#[derive(Debug, PartialEq, Eq, Hash)]
struct Checksum([u8; 32]);

impl TryFrom<&[u8]> for Checksum {
    type Error = eyre::Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Checksum(slice_to_array::<32>(value)?))
    }
}
use std::fmt;

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", const_hex::encode(&self.0[..]))
    }
}

fn tempfile(out_dir: &Path) -> Result<NamedTempFile> {
    let t = NamedTempFile::new_in(out_dir)?;
    Ok(t)
}

/// Download the file piecewise to a temporary file and calculate the digest
async fn download_builtin_actors_bundle(
    tag: impl AsRef<str>,
    out_dir: &Path,
) -> Result<(NamedTempFile, Checksum)> {
    let tag = tag.as_ref();
    let mut tmp = tempfile(out_dir)?;

    let url = format!("https://github.com/filecoin-project/builtin-actors/releases/download/{tag}/builtin-actors-mainnet.car");
    let url = reqwest::Url::parse(&url)?;

    let response = reqwest::get(url).await?;
    let mut stream = response.bytes_stream();
    // concurrently compute the sha2 & write to temp file
    let mut sha = sha2::Sha256::default();
    while let Some(piece) = stream.next().await {
        let piece = piece?;
        let piece = piece.slice(..);
        let mut reader = piece.clone().reader();
        std::io::copy(&mut reader, &mut sha)?;

        let mut reader = piece.clone().reader();
        std::io::copy(&mut reader, &mut tmp.as_file_mut())?;
    }
    let digest = sha.finalize();
    let digest = Checksum::try_from(digest.as_slice())?;
    Ok((tmp, digest))
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    echo!("builtin", blue, "Fetching builtin filecoin actors");

    let out_dir = std::env::var("OUT_DIR").wrap_err("Missing OUT_DIR env")?;
    let out_dir = std::path::PathBuf::from(out_dir);

    let builtin_car_path = out_dir.join("builtin_actors.car");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed={}", FORCE_RERUN);
    println!("cargo::rerun-if-env-changed={}", VERSION_OVERRIDE);

    println!("cargo::rerun-if-changed={}", builtin_car_path.display());

    let tag = std::env::var(VERSION_OVERRIDE).unwrap_or_else(|_e| BUILTIN_ACTORS_TAG.to_owned());

    let download = match fs::File::open(&builtin_car_path) {
        Ok(f) => {
            // compare digests, if mismatch, replace existing with the downloaded file
            let actual = file_digest(f)?;
            if BUILTIN_ACTORS_SHA256SUM != actual.to_string() {
                echo!(
                    "builtin",
                    blue,
                    "Builtin actors local file {} mismatched expected digest {}, re-downloading..",
                    actual,
                    BUILTIN_ACTORS_SHA256SUM
                );
                fs::remove_file(&builtin_car_path)?;
                true
            } else {
                echo!(
                    "builtin",
                    blue,
                    "Nothing to do, identical file is already present"
                );
                false
            }
        }
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                // the file is not found, so a simple rename is good enough
                true
            } else {
                // other errors always must lead to an error
                bail!(e)
            }
        }
    };
    if download {
        let (tmp, tmp_digest) = download_builtin_actors_bundle(tag, &out_dir).await?;
        if tmp_digest.to_string() != BUILTIN_ACTORS_SHA256SUM {
            bail!("Mismatch of hardcoded sha256sum and actual digest")
        }
        fs::rename(tmp.path(), &builtin_car_path)?;
    }

    echo!(
        "builtin",
        red,
        "Builtin actors file is ready for inclusion: {}",
        builtin_car_path.display()
    );

    Ok(())
}
