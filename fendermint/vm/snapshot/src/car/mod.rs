// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! CAR file chunking utilities
//!
//! See https://ipld.io/specs/transport/car/carv1/

use anyhow::{self, Context as AnyhowContext};
use futures::{future, io::Cursor, StreamExt};
use std::path::Path;

use fvm_ipld_car::{CarHeader, CarReader};

use self::{chunker::ChunkWriter, streamer::BlockStreamer};

mod chunker;
mod streamer;

/// Take an existing CAR file and split it up into an output directory by creating
/// files with a limited size for each file.
///
/// The first (0th) file will be just the header, with the rest containing the "content" blocks.
///
/// Returns the number of chunks created.
pub async fn split<F>(
    input_car: impl Into<std::borrow::Cow<'static, [u8]>>,
    output_dir: &'_ Path,
    max_size: usize,
    file_name: F,
) -> anyhow::Result<usize>
where
    F: Fn(usize) -> String,
    F: Send + Sync + 'static,
{
    let input_car = input_car.into();
    let output_dir = output_dir.to_path_buf();

    let input_car = Cursor::new(input_car);
    let reader: CarReader<_> = CarReader::new_unchecked(input_car)
        .await
        .context("failed to open CAR reader")?;

    // Create a Writer that opens new files when the maximum is reached.
    let mut writer = ChunkWriter::new(output_dir, max_size, file_name);

    let header = CarHeader::new(reader.header.roots.clone(), reader.header.version);

    let block_streamer = BlockStreamer::new(reader);
    // We shouldn't see errors when reading the CAR files, as we have written them ourselves,
    // but for piece of mind let's log any errors and move on.
    let mut block_streamer = block_streamer.filter_map(|res| match res {
        Ok(b) => future::ready(Some(b)),
        Err(e) => {
            // TODO: It would be better to stop if there are errors.
            tracing::warn!(error = e.to_string(), "CAR block failure");
            future::ready(None)
        }
    });

    // Copy the input CAR into an output CAR.
    header
        .write_stream_async(&mut writer, &mut block_streamer)
        .await
        .context("failed to write CAR file")?;

    Ok(writer.chunk_created())
}

#[cfg(test)]
mod tests {
    use fs_err as fs;

    use tempfile::tempdir;

    use super::split;

    /// Load the actor bundle CAR file, split it into chunks, then restore and compare to the original.
    #[tokio::test]
    async fn split_bundle_car() {
        let bundle_bytes = actors_custom_car::CAR;

        let tmp = tempdir().unwrap();
        let target_count = 10;
        let max_size = bundle_bytes.len() / target_count;

        let chunks_count = split(bundle_bytes, tmp.path(), max_size, |idx| idx.to_string())
            .await
            .expect("failed to split CAR file");

        let mut chunks = fs::read_dir(tmp.path())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // There are few enough that we can get away without converting to an integer.
        chunks.sort_unstable_by_key(|c| c.path().to_string_lossy().to_string());

        let chunks = chunks
            .into_iter()
            .map(|c| {
                let chunk_size = fs::metadata(c.path()).unwrap().len() as usize;
                (c, chunk_size)
            })
            .collect::<Vec<_>>();

        let chunks_bytes = chunks.iter().fold(Vec::new(), |mut acc, (c, _)| {
            let bz = fs::read(c.path()).unwrap();
            acc.extend(bz);
            acc
        });

        assert_eq!(chunks_count, chunks.len());

        assert!(
            1 < chunks.len() && chunks.len() <= 1 + target_count,
            "expected 1 header and max {} chunks, got {}",
            target_count,
            chunks.len()
        );

        assert!(chunks[0].1 < 100, "header is small");
        assert_eq!(chunks_bytes.len(), bundle_bytes.len());
        assert_eq!(chunks_bytes[0..100], bundle_bytes[0..100]);
    }
}
