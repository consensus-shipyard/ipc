// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: MIT

mod types;

pub(crate) use crate::manager::cometbft::types::SignedHeader;
use fvm_shared::clock::ChainEpoch;
use tendermint_rpc::Client;

pub struct CometbftClient {
    client: tendermint_rpc::HttpClient,
}

impl CometbftClient {
    pub fn new_from_url(url: &str) -> Self {
        let client =
            tendermint_rpc::HttpClient::new(url).expect("could not create tendermint client");
        Self { client }
    }

    pub async fn fetch_signed_header(&self, height: ChainEpoch) -> anyhow::Result<SignedHeader> {
        let h = tendermint::block::Height::from(height as u32);
        let query_response = self.client.commit(h).await?;
        Ok(SignedHeader::from(query_response.signed_header))
    }
}
