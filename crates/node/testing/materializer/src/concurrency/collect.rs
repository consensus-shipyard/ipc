// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ethers::prelude::H256;
use ethers::providers::Http;
use ethers::providers::{Middleware, Provider};
use ethers::types::Block;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

pub async fn collect_blocks<F>(
    token: CancellationToken,
    provider: Provider<Http>,
    assert: F,
) -> anyhow::Result<HashMap<u64, Block<H256>>>
where
    F: Fn(&Block<H256>),
{
    let mut blocks = HashMap::new();
    while !token.is_cancelled() {
        // TODO: improve: use less calls, make sure blocks cannot be missed
        let block_number = provider.get_block_number().await?;
        let block = provider.get_block(block_number).await?.unwrap();
        assert(&block);
        blocks.insert(block_number.as_u64(), block);

        sleep(Duration::from_millis(100)).await;
    }
    Ok(blocks)
}
