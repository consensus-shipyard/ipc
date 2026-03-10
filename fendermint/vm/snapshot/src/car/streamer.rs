// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

use fvm_ipld_car::CarReader;
use fvm_ipld_car::Error as CarError;

type BlockStreamerItem = Result<(Cid, Vec<u8>), CarError>;

/// Stream the content blocks from a CAR reader.
/// In FVM 4.7, CarReader is a synchronous iterator, so we wrap it for async use
pub struct BlockStreamer<R> {
    reader: Option<CarReader<R>>,
}

impl<R> BlockStreamer<R>
where
    R: std::io::Read + Send,
{
    pub fn new(reader: CarReader<R>) -> Self {
        Self {
            reader: Some(reader),
        }
    }
}

impl<R> Stream for BlockStreamer<R>
where
    R: std::io::Read + Send + Unpin + 'static,
{
    type Item = BlockStreamerItem;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // In FVM 4.7, CarReader is a synchronous iterator
        if let Some(ref mut reader) = self.reader {
            match reader.next() {
                Some(Ok(block)) => Poll::Ready(Some(Ok((block.cid, block.data)))),
                Some(Err(e)) => Poll::Ready(Some(Err(e))),
                None => {
                    self.reader = None;
                    Poll::Ready(None)
                }
            }
        } else {
            Poll::Ready(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use fvm_ipld_blockstore::MemoryBlockstore;
    use fvm_ipld_car::{load_car, CarReader};

    use super::BlockStreamer;

    /// Check that a CAR file can be loaded from a byte reader.
    fn check_load_car(reader: impl std::io::Read) {
        let store = MemoryBlockstore::new();
        load_car(&store, reader).expect("failed to load CAR");
    }

    /// Check that a CAR file can be streamed without errors.
    async fn check_block_streamer(reader: impl std::io::Read + Send + Unpin + 'static) {
        let reader = CarReader::new(reader).expect("failed to open CAR reader");

        let streamer = BlockStreamer::new(reader);

        streamer
            .for_each(|r| async move {
                r.expect("should be ok");
            })
            .await;
    }

    /// Sanity check that the test bundle can be loaded with the normal facilities from a file.
    #[tokio::test]
    async fn load_bundle_from_file() {
        let car_bundle = actors_custom_car::CAR;
        check_load_car(car_bundle);
    }

    #[tokio::test]
    async fn block_streamer_from_file() {
        let bundle_file = actors_custom_car::CAR;
        check_block_streamer(bundle_file).await;
    }
}
