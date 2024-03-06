// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::sync::{Arc, Mutex};

use crate::state::ActorType;
use anyhow::Context;
use cid::Cid;
use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::query::QueryClient;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_shared::{
    address::{Address, Payload},
    ActorID,
};
use lru_time_cache::LruCache;
use tendermint_rpc::Client;

/// Facilitate Ethereum address <-> Actor ID lookups.
#[derive(Clone)]
pub struct AddressCache<C> {
    client: FendermintClient<C>,
    addr_to_id: Arc<Mutex<LruCache<Address, ActorID>>>,
    id_to_addr: Arc<Mutex<LruCache<ActorID, Address>>>,
    addr_to_actor_type: Arc<Mutex<LruCache<Address, ActorType>>>,
    cid_to_actor_type: Arc<Mutex<LruCache<Cid, ActorType>>>,
}

impl<C> AddressCache<C>
where
    C: Client + Sync + Send,
{
    pub fn new(client: FendermintClient<C>, capacity: usize) -> Self {
        Self {
            client,
            addr_to_id: Arc::new(Mutex::new(LruCache::with_capacity(capacity))),
            id_to_addr: Arc::new(Mutex::new(LruCache::with_capacity(capacity))),
            addr_to_actor_type: Arc::new(Mutex::new(LruCache::with_capacity(capacity))),
            cid_to_actor_type: Arc::new(Mutex::new(LruCache::with_capacity(capacity))),
        }
    }

    pub async fn lookup_id(&self, addr: &Address) -> anyhow::Result<Option<ActorID>> {
        if let Ok(id) = addr.id() {
            return Ok(Some(id));
        }

        if let Some(id) = self.get_id(addr) {
            return Ok(Some(id));
        }

        // Using committed height because pending could change.
        let res = self
            .client
            .actor_state(addr, FvmQueryHeight::Committed)
            .await
            .context("failed to lookup actor state")?;

        if let Some((id, _)) = res.value {
            self.set_id(*addr, id);
            if let Payload::Delegated(_) = addr.payload() {
                self.set_addr(id, *addr)
            }
            return Ok(Some(id));
        }
        tracing::info!(
            addr = addr.to_string(),
            height = res.height.value(),
            "actor not found"
        );
        Ok(None)
    }

    /// Look up the delegated address of an ID, if any.
    pub async fn lookup_addr(&self, id: &ActorID) -> anyhow::Result<Option<Address>> {
        if let Some(addr) = self.get_addr(id) {
            return Ok(Some(addr));
        }

        let res = self
            .client
            .actor_state(&Address::new_id(*id), FvmQueryHeight::Committed)
            .await
            .context("failed to lookup actor state")?;

        if let Some((_, actor_state)) = res.value {
            if let Some(addr) = actor_state.delegated_address {
                self.set_addr(*id, addr);
                self.set_id(addr, *id);
                return Ok(Some(addr));
            }
        }
        tracing::info!(id, height = res.height.value(), "actor not found");
        Ok(None)
    }

    fn get_id(&self, addr: &Address) -> Option<ActorID> {
        let mut c = self.addr_to_id.lock().unwrap();
        c.get(addr).cloned()
    }

    fn set_id(&self, addr: Address, id: ActorID) {
        let mut c = self.addr_to_id.lock().unwrap();
        c.insert(addr, id);
    }

    fn get_addr(&self, id: &ActorID) -> Option<Address> {
        let mut c = self.id_to_addr.lock().unwrap();
        c.get(id).cloned()
    }

    fn set_addr(&self, id: ActorID, addr: Address) {
        let mut c = self.id_to_addr.lock().unwrap();
        c.insert(id, addr);
    }

    pub fn set_actor_type_for_addr(&self, addr: Address, actor_type: ActorType) {
        let mut c = self.addr_to_actor_type.lock().unwrap();
        c.insert(addr, actor_type);
    }

    pub fn get_actor_type_from_addr(&self, addr: &Address) -> Option<ActorType> {
        let mut c = self.addr_to_actor_type.lock().unwrap();
        c.get(addr).cloned()
    }

    pub fn set_actor_type_for_cid(&self, cid: Cid, actor_type: ActorType) {
        let mut c = self.cid_to_actor_type.lock().unwrap();
        c.insert(cid, actor_type);
    }

    pub fn get_actor_type_from_cid(&self, cid: &Cid) -> Option<ActorType> {
        let mut c = self.cid_to_actor_type.lock().unwrap();
        c.get(cid).cloned()
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::AddressCache;
    use crate::state::ActorType;
    use cid::Cid;
    use fendermint_rpc::FendermintClient;
    use fvm_shared::address::Address;
    use std::str::FromStr;
    use tendermint_rpc::MockClient;

    #[test]
    fn test_read_and_write_addr_to_actor_type() {
        let client = FendermintClient::new(
            MockClient::new(tendermint_rpc::MockRequestMethodMatcher::default()).0,
        );
        let addr_cache = AddressCache::new(client, 1000);

        let address1 = Address::from_str("f410fivboj67m6ut4j6xx3lhc426io22r7l3h6yha5bi").unwrap();
        let address2 = Address::from_str("f410fmpohbjcmznke3e7pbxomsbg5uae6o2sfjurchxa").unwrap();
        let address3 = Address::from_str("f410fxbfwpcrgbjg2ab6fevpoi4qlcfosw2vk5kzo5ga").unwrap();
        let address4 = Address::from_str("f410fggjevhgketpz6gw6ordusynlgcd5piyug4aomuq").unwrap();

        addr_cache.set_actor_type_for_addr(address1, ActorType::EVM);
        addr_cache.set_actor_type_for_addr(address2, ActorType::Unknown(Cid::default()));
        addr_cache.set_actor_type_for_addr(address3, ActorType::Inexistent);

        assert_eq!(
            addr_cache.get_actor_type_from_addr(&address1).unwrap(),
            ActorType::EVM
        );
        assert_eq!(
            addr_cache.get_actor_type_from_addr(&address2).unwrap(),
            ActorType::Unknown(Cid::default())
        );
        assert_eq!(
            addr_cache.get_actor_type_from_addr(&address3).unwrap(),
            ActorType::Inexistent
        );
        assert_eq!(addr_cache.get_actor_type_from_addr(&address4), None);
    }

    #[test]
    fn test_read_and_write_cid_to_actor_type() {
        let client = FendermintClient::new(
            MockClient::new(tendermint_rpc::MockRequestMethodMatcher::default()).0,
        );
        let addr_cache = AddressCache::new(client, 1000);

        let cid1 = Cid::from_str("bafk2bzacecmnyfiwb52tkbwmm2dsd7ysi3nvuxl3lmspy7pl26wxj4zj7w4wi")
            .unwrap();
        let cid2 = Cid::from_str("bafy2bzaceas2zajrutdp7ugb6w2lpmow3z3klr3gzqimxtuz22tkkqallfch4")
            .unwrap();
        let cid3 = Cid::from_str("k51qzi5uqu5dlvj2baxnqndepeb86cbk3ng7n3i46uzyxzyqj2xjonzllnv0v8")
            .unwrap();
        let cid4 =
            Cid::from_str("bafybeiemxf5abjwjbikoz4mc3a3dla6ual3jsgpdr4cjr3oz3evfyavhwq").unwrap();

        addr_cache.set_actor_type_for_cid(cid1, ActorType::EVM);
        addr_cache.set_actor_type_for_cid(cid2, ActorType::Unknown(Cid::default()));
        addr_cache.set_actor_type_for_cid(cid3, ActorType::Inexistent);

        assert_eq!(
            addr_cache.get_actor_type_from_cid(&cid1).unwrap(),
            ActorType::EVM
        );
        assert_eq!(
            addr_cache.get_actor_type_from_cid(&cid2).unwrap(),
            ActorType::Unknown(Cid::default())
        );
        assert_eq!(
            addr_cache.get_actor_type_from_cid(&cid3).unwrap(),
            ActorType::Inexistent
        );
        assert_eq!(addr_cache.get_actor_type_from_cid(&cid4), None);
    }
}
