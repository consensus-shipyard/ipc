// Copyright 2024 Hoku Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fendermint_actor_blobs_shared::state::Account;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use hoku_ipld::map::{Map, DEFAULT_HAMT_CONFIG};

pub type AccountMap<BS> = Map<BS, Address, Account>;

pub struct Accounts<BS: Blockstore> {
    pub map: AccountMap<BS>,
}

impl<'a, BS> Accounts<BS>
where
    BS: Blockstore,
{
    pub fn flush_empty(store: BS) -> Result<Cid, ActorError> {
        AccountMap::flush_empty(store, DEFAULT_HAMT_CONFIG)
    }

    pub fn load(store: &'a BS, root: &Cid) -> Result<Accounts<&'a BS>, ActorError> {
        let map = AccountMap::load(store, root, DEFAULT_HAMT_CONFIG, "accounts")?;
        Ok(Accounts { map })
    }

    pub fn get(&self, addr: &Address) -> Result<Option<Account>, ActorError> {
        self.map.get(addr).map(|a| a.cloned())
    }

    pub fn get_or_err(&self, addr: &Address) -> Result<Account, ActorError> {
        self.get(addr)?
            .ok_or(ActorError::not_found(format!("account {} not found", addr)))
    }

    pub fn get_or_create(
        &self,
        addr: &Address,
        current_epoch: ChainEpoch,
    ) -> Result<Account, ActorError> {
        if let Some(a) = self.map.get(addr)? {
            Ok(a.clone())
        } else {
            Ok(Account::new(current_epoch))
        }
    }

    pub fn set_and_flush(&mut self, addr: &Address, account: Account) -> Result<Cid, ActorError> {
        self.map.set(addr, account)?;
        self.map.flush()
    }

    /// Consumes the underlying map's HAMT and returns the Blockstore it owns.
    pub fn into_store(self) -> BS {
        self.map.into_store()
    }
}
