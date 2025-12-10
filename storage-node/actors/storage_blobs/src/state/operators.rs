// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch};
use storage_node_ipld::hamt::{self, map::TrackedFlushResult};

/// Information about a registered node operator
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct NodeOperatorInfo {
    /// BLS public key (48 bytes)
    pub bls_pubkey: Vec<u8>,

    /// RPC URL for gateway to query signatures
    pub rpc_url: String,

    /// Epoch when operator registered
    pub registered_epoch: ChainEpoch,

    /// Whether operator is active
    pub active: bool,
}

/// Registry of node operators
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Operators {
    /// HAMT root: Address → NodeOperatorInfo
    pub root: hamt::Root<Address, NodeOperatorInfo>,

    /// Ordered list of active operator addresses
    /// Index in this vec = bit position in bitmap for signature aggregation
    pub active_list: Vec<Address>,

    /// Total number of registered operators
    size: u64,
}

impl Operators {
    /// Creates a new empty [`Operators`] registry
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Address, NodeOperatorInfo>::new(store, "operators")?;
        Ok(Self {
            root,
            active_list: Vec::new(),
            size: 0,
        })
    }

    /// Returns the underlying [`hamt::map::Hamt`]
    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, Address, NodeOperatorInfo>, ActorError> {
        self.root.hamt(store, self.size)
    }

    /// Saves the state from the [`TrackedFlushResult`]
    pub fn save_tracked(
        &mut self,
        tracked_flush_result: TrackedFlushResult<Address, NodeOperatorInfo>,
    ) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    /// Returns the number of registered operators
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Returns true if there are no registered operators
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Register a new operator (adds to end of active_list)
    /// Returns the operator's index in the active_list
    pub fn register<BS: Blockstore>(
        &mut self,
        store: BS,
        address: Address,
        info: NodeOperatorInfo,
    ) -> Result<usize, ActorError> {
        let mut hamt = self.hamt(store)?;

        // Check if operator already exists
        if hamt.get(&address)?.is_some() {
            return Err(ActorError::illegal_argument(
                "Operator already registered".into(),
            ));
        }

        // Add to HAMT
        self.save_tracked(hamt.set_and_flush_tracked(&address, info)?);

        // Add to active list (gets next available index)
        let index = self.active_list.len();
        self.active_list.push(address);

        Ok(index)
    }

    /// Get operator info by address
    pub fn get<BS: Blockstore>(
        &self,
        store: BS,
        address: &Address,
    ) -> Result<Option<NodeOperatorInfo>, ActorError> {
        self.hamt(store)?.get(address)
    }

    /// Get operator index in active_list (for bitmap generation)
    /// Returns None if operator is not in the active list
    pub fn get_index(&self, address: &Address) -> Option<usize> {
        self.active_list.iter().position(|a| a == address)
    }

    /// Get all active operators in order
    pub fn get_active_operators(&self) -> Vec<Address> {
        self.active_list.clone()
    }

    /// Update operator info (e.g., to change RPC URL or deactivate)
    pub fn update<BS: Blockstore>(
        &mut self,
        store: BS,
        address: &Address,
        info: NodeOperatorInfo,
    ) -> Result<(), ActorError> {
        let mut hamt = self.hamt(store)?;

        // Check if operator exists
        if hamt.get(address)?.is_none() {
            return Err(ActorError::not_found("Operator not found".into()));
        }

        // Update in HAMT
        self.save_tracked(hamt.set_and_flush_tracked(address, info)?);

        Ok(())
    }

    /// Deactivate an operator (removes from active_list but keeps in HAMT)
    /// Note: This will change indices of all operators after the removed one
    pub fn deactivate<BS: Blockstore>(
        &mut self,
        store: BS,
        address: &Address,
    ) -> Result<(), ActorError> {
        let mut hamt = self.hamt(store)?;

        // Get existing info
        let mut info = hamt
            .get(address)?
            .ok_or_else(|| ActorError::not_found("Operator not found".into()))?;

        // Mark as inactive
        info.active = false;
        self.save_tracked(hamt.set_and_flush_tracked(address, info)?);

        // Remove from active_list
        if let Some(pos) = self.active_list.iter().position(|a| a == address) {
            self.active_list.remove(pos);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fvm_ipld_blockstore::MemoryBlockstore;

    fn new_test_address(id: u64) -> Address {
        Address::new_id(id)
    }

    fn new_test_operator(pubkey: u8) -> NodeOperatorInfo {
        NodeOperatorInfo {
            bls_pubkey: vec![pubkey; 48],
            rpc_url: format!("http://operator{}.example.com:8080", pubkey),
            registered_epoch: 0,
            active: true,
        }
    }

    #[test]
    fn test_register_operator() {
        let store = MemoryBlockstore::default();
        let mut operators = Operators::new(&store).unwrap();

        let addr1 = new_test_address(100);
        let info1 = new_test_operator(1);

        let index = operators.register(&store, addr1, info1.clone()).unwrap();
        assert_eq!(index, 0);
        assert_eq!(operators.len(), 1);

        let retrieved = operators.get(&store, &addr1).unwrap().unwrap();
        assert_eq!(retrieved, info1);
    }

    #[test]
    fn test_active_list_ordering() {
        let store = MemoryBlockstore::default();
        let mut operators = Operators::new(&store).unwrap();

        let addr1 = new_test_address(100);
        let addr2 = new_test_address(101);
        let addr3 = new_test_address(102);

        operators
            .register(&store, addr1, new_test_operator(1))
            .unwrap();
        operators
            .register(&store, addr2, new_test_operator(2))
            .unwrap();
        operators
            .register(&store, addr3, new_test_operator(3))
            .unwrap();

        assert_eq!(operators.get_index(&addr1), Some(0));
        assert_eq!(operators.get_index(&addr2), Some(1));
        assert_eq!(operators.get_index(&addr3), Some(2));

        let active = operators.get_active_operators();
        assert_eq!(active, vec![addr1, addr2, addr3]);
    }

    #[test]
    fn test_duplicate_registration() {
        let store = MemoryBlockstore::default();
        let mut operators = Operators::new(&store).unwrap();

        let addr1 = new_test_address(100);
        operators
            .register(&store, addr1, new_test_operator(1))
            .unwrap();

        let result = operators.register(&store, addr1, new_test_operator(2));
        assert!(result.is_err());
    }

    #[test]
    fn test_deactivate_operator() {
        let store = MemoryBlockstore::default();
        let mut operators = Operators::new(&store).unwrap();

        let addr1 = new_test_address(100);
        let addr2 = new_test_address(101);
        let addr3 = new_test_address(102);

        operators
            .register(&store, addr1, new_test_operator(1))
            .unwrap();
        operators
            .register(&store, addr2, new_test_operator(2))
            .unwrap();
        operators
            .register(&store, addr3, new_test_operator(3))
            .unwrap();

        // Deactivate middle operator
        operators.deactivate(&store, &addr2).unwrap();

        // Check active list updated
        let active = operators.get_active_operators();
        assert_eq!(active, vec![addr1, addr3]);

        // Check indices shifted
        assert_eq!(operators.get_index(&addr1), Some(0));
        assert_eq!(operators.get_index(&addr2), None);
        assert_eq!(operators.get_index(&addr3), Some(1));

        // Check still in HAMT but marked inactive
        let info = operators.get(&store, &addr2).unwrap().unwrap();
        assert!(!info.active);
    }
}
