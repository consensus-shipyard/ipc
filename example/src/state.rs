use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::Cbor;
use fvm_ipld_hamt::BytesKey;
use fvm_shared::address::Address;
use serde::{Deserialize, Serialize};
use tcid::{TCid, THamt};

/// Sample struct for user persistence
#[derive(Serialize, Deserialize)]
pub struct UserPersistParam {
    pub name: String,
}

/// User data storage struct
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct User {
    pub name: String,
    pub owner: Address,
}

/// The state storage struct, persisted in BlockStore
#[derive(Serialize, Deserialize)]
pub struct State {
    pub call_count: usize,
    pub typed_hamt: TCid<THamt<Cid, User>>,
}

impl Cbor for State {}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self> {
        Ok(State {
            call_count: 0,
            typed_hamt: TCid::new_hamt(store)?,
        })
    }

    pub fn upsert_user<BS: Blockstore>(
        &mut self,
        address: &Address,
        name: String,
        store: &BS,
    ) -> anyhow::Result<()> {
        let key = BytesKey::from(address.to_bytes());
        let mut hamt = self.typed_hamt.load(store)?;
        hamt.set(
            key,
            User {
                owner: *address,
                name,
            },
        )?;

        self.call_count += 1;
        self.typed_hamt.flush(hamt)?;

        Ok(())
    }
}
