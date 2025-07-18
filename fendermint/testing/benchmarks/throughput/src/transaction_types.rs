// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Transaction type implementations for different workloads

use std::sync::Arc;
use anyhow::Result;
use ethers::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Simple transfer transaction generator
pub struct TransferGenerator {
    provider: Arc<Provider<Http>>,
    accounts: Vec<LocalWallet>,
    value: U256,
}

impl TransferGenerator {
    pub fn new(provider: Arc<Provider<Http>>, accounts: Vec<LocalWallet>, value: u64) -> Self {
        Self {
            provider,
            accounts,
            value: U256::from(value),
        }
    }

    pub async fn generate_transaction(&self) -> Result<TransactionRequest> {
        let mut rng = rand::thread_rng();
        let recipient_idx = rng.gen_range(0..self.accounts.len());
        let recipient = self.accounts[recipient_idx].address();

        Ok(TransactionRequest::new()
            .to(recipient)
            .value(self.value)
            .gas(21000))
    }
}

/// ERC-20 token transfer generator
pub struct Erc20Generator {
    provider: Arc<Provider<Http>>,
    accounts: Vec<LocalWallet>,
    token_address: Address,
    amount: U256,
}

impl Erc20Generator {
    pub fn new(
        provider: Arc<Provider<Http>>,
        accounts: Vec<LocalWallet>,
        token_address: Address,
        amount: u64,
    ) -> Self {
        Self {
            provider,
            accounts,
            token_address,
            amount: U256::from(amount),
        }
    }

    pub async fn generate_transaction(&self) -> Result<TransactionRequest> {
        let mut rng = rand::thread_rng();
        let recipient_idx = rng.gen_range(0..self.accounts.len());
        let recipient = self.accounts[recipient_idx].address();

        // ERC-20 transfer function signature: transfer(address,uint256)
        let function_selector = [0xa9, 0x05, 0x9c, 0xbb]; // Keccak256("transfer(address,uint256)")[:4]
        let mut data = function_selector.to_vec();

        // Encode recipient address (32 bytes)
        data.extend_from_slice(&[0u8; 12]); // Padding
        data.extend_from_slice(recipient.as_bytes());

        // Encode amount (32 bytes)
        let mut amount_bytes = [0u8; 32];
        self.amount.to_big_endian(&mut amount_bytes);
        data.extend_from_slice(&amount_bytes);

        Ok(TransactionRequest::new()
            .to(self.token_address)
            .data(data)
            .gas(60000))
    }
}

/// Contract deployment generator
pub struct DeployGenerator {
    provider: Arc<Provider<Http>>,
    contract_bytecode: Vec<u8>,
}

impl DeployGenerator {
    pub fn new(provider: Arc<Provider<Http>>, contract_bytecode: Vec<u8>) -> Self {
        Self {
            provider,
            contract_bytecode,
        }
    }

    pub async fn generate_transaction(&self) -> Result<TransactionRequest> {
        Ok(TransactionRequest::new()
            .data(self.contract_bytecode.clone())
            .gas(500000))
    }
}

/// Simple contract call generator
pub struct ContractCallGenerator {
    provider: Arc<Provider<Http>>,
    contract_address: Address,
    call_data: Vec<u8>,
}

impl ContractCallGenerator {
    pub fn new(provider: Arc<Provider<Http>>, contract_address: Address, call_data: Vec<u8>) -> Self {
        Self {
            provider,
            contract_address,
            call_data,
        }
    }

    pub async fn generate_transaction(&self) -> Result<TransactionRequest> {
        Ok(TransactionRequest::new()
            .to(self.contract_address)
            .data(self.call_data.clone())
            .gas(100000))
    }
}

/// Common contract bytecodes for testing
pub struct TestContracts;

impl TestContracts {
    /// Simple storage contract that increments a counter
    pub fn counter_contract() -> Vec<u8> {
        hex::decode("608060405234801561001057600080fd5b50600080819055506101a8806100276000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c806306661abd1461003b578063d14e62b814610055575b600080fd5b610043610073565b60405190815260200160405180910390f35b61005d610079565b60405190815260200160405180910390f35b60005481565b600080549050600160008282546100909190610094565b9091555050600054905090565b600082198211156100b157634e487b7160e01b600052601160045260246000fd5b50019056fea26469706673582212200000000000000000000000000000000000000000000000000000000000000000064736f6c634300080a0033")
            .unwrap_or_else(|_| vec![0x60, 0x80, 0x60, 0x40, 0x52, 0x00]) // Fallback minimal bytecode
    }

    /// Simple ERC-20 token contract
    pub fn erc20_contract() -> Vec<u8> {
        hex::decode("608060405234801561001057600080fd5b506040805190810160405280600481526020017f54657374000000000000000000000000000000000000000000000000000000008152506040805190810160405280600481526020017f54535400000000000000000000000000000000000000000000000000000000008152508160039080519060200190610098929190610140565b5080600490805190602001906100af929190610140565b506012600560006101000a81548160ff021916908360ff16021790555050505050610000565b828054600181600116156101000203166002900490600052602060002090601f016020900481019282601f1061018157805160ff19168380011785556101af565b828001600101855582156101af579182015b828111156101ae578251825591602001919060010190610193565b5b5090506101bc91906101c0565b5090565b6101e291905b808211156101de5760008160009055506001016101c6565b5090565b90565b6000000000000000000000000000000000000000000000000000000000000000")
            .unwrap_or_else(|_| vec![0x60, 0x80, 0x60, 0x40, 0x52, 0x00]) // Fallback minimal bytecode
    }

    /// Empty contract (minimal deployment)
    pub fn empty_contract() -> Vec<u8> {
        hex::decode("6080604052348015600f57600080fd5b50603f80601d6000396000f3fe6080604052600080fdfea264697066735822122000000000000000000000000000000000000000000000000000000000000000064736f6c634300080a0033")
            .unwrap_or_else(|_| vec![0x60, 0x80, 0x60, 0x40, 0x52, 0x00]) // Fallback minimal bytecode
    }

    /// Function selector for incrementing counter
    pub fn counter_increment_selector() -> Vec<u8> {
        vec![0xd1, 0x4e, 0x62, 0xb8] // increment()
    }

    /// Function selector for getting counter value
    pub fn counter_get_selector() -> Vec<u8> {
        vec![0x06, 0x66, 0x1a, 0xbd] // get()
    }
}

/// Transaction template for different workload patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionTemplate {
    pub name: String,
    pub transaction_type: crate::TransactionType,
    pub gas_limit: u64,
    pub value: u64,
    pub data: Option<Vec<u8>>,
    pub to: Option<String>,
    pub weight: f64, // Weight in mixed workloads
}

impl TransactionTemplate {
    /// Create a simple transfer template
    pub fn transfer(value: u64) -> Self {
        Self {
            name: "Simple Transfer".to_string(),
            transaction_type: crate::TransactionType::Transfer,
            gas_limit: 21000,
            value,
            data: None,
            to: None,
            weight: 1.0,
        }
    }

    /// Create an ERC-20 transfer template
    pub fn erc20_transfer(token_address: &str, amount: u64) -> Self {
        Self {
            name: "ERC-20 Transfer".to_string(),
            transaction_type: crate::TransactionType::Erc20,
            gas_limit: 60000,
            value: 0,
            data: Some(Self::encode_erc20_transfer(amount)),
            to: Some(token_address.to_string()),
            weight: 1.0,
        }
    }

    /// Create a contract deployment template
    pub fn deploy(contract_bytecode: Vec<u8>) -> Self {
        Self {
            name: "Contract Deployment".to_string(),
            transaction_type: crate::TransactionType::Deploy,
            gas_limit: 500000,
            value: 0,
            data: Some(contract_bytecode),
            to: None,
            weight: 0.1, // Deployments are typically less frequent
        }
    }

    /// Create a contract call template
    pub fn contract_call(contract_address: &str, call_data: Vec<u8>) -> Self {
        Self {
            name: "Contract Call".to_string(),
            transaction_type: crate::TransactionType::ContractCall,
            gas_limit: 100000,
            value: 0,
            data: Some(call_data),
            to: Some(contract_address.to_string()),
            weight: 0.5,
        }
    }

    /// Encode ERC-20 transfer function call
    fn encode_erc20_transfer(amount: u64) -> Vec<u8> {
        let mut data = vec![0xa9, 0x05, 0x9c, 0xbb]; // transfer(address,uint256)
        data.extend_from_slice(&[0u8; 32]); // Recipient (will be set randomly)

        let mut amount_bytes = [0u8; 32];
        U256::from(amount).to_big_endian(&mut amount_bytes);
        data.extend_from_slice(&amount_bytes);

        data
    }
}

/// Workload pattern that combines multiple transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadPattern {
    pub name: String,
    pub templates: Vec<TransactionTemplate>,
    pub description: String,
}

impl WorkloadPattern {
    /// Create a simple transfer workload
    pub fn simple_transfers() -> Self {
        Self {
            name: "Simple Transfers".to_string(),
            templates: vec![TransactionTemplate::transfer(1000000000000000000)], // 1 ETH
            description: "100% simple FIL transfers".to_string(),
        }
    }

    /// Create a DeFi-like workload pattern
    pub fn defi_workload() -> Self {
        Self {
            name: "DeFi Workload".to_string(),
            templates: vec![
                TransactionTemplate::transfer(1000000000000000000), // 1 ETH transfers
                TransactionTemplate::erc20_transfer("0x1234567890123456789012345678901234567890", 1000000), // Token transfers
                TransactionTemplate::contract_call("0x1234567890123456789012345678901234567890", vec![0x70, 0xa0, 0x82, 0x31]), // swap()
            ],
            description: "Mixed DeFi transactions: 50% transfers, 30% ERC-20, 20% DEX swaps".to_string(),
        }
    }

    /// Create a gaming workload pattern
    pub fn gaming_workload() -> Self {
        Self {
            name: "Gaming Workload".to_string(),
            templates: vec![
                TransactionTemplate::transfer(1000000000000000), // 0.001 ETH micro-payments
                TransactionTemplate::contract_call("0x1234567890123456789012345678901234567890", vec![0x12, 0x34, 0x56, 0x78]), // game_action()
                TransactionTemplate::contract_call("0x1234567890123456789012345678901234567890", vec![0x87, 0x65, 0x43, 0x21]), // claim_reward()
            ],
            description: "Gaming transactions: 60% game actions, 30% micro-payments, 10% rewards".to_string(),
        }
    }

    /// Create an NFT marketplace workload
    pub fn nft_workload() -> Self {
        Self {
            name: "NFT Marketplace".to_string(),
            templates: vec![
                TransactionTemplate::transfer(100000000000000000), // 0.1 ETH
                TransactionTemplate::contract_call("0x1234567890123456789012345678901234567890", vec![0xa2, 0x2c, 0xb4, 0x65]), // mint()
                TransactionTemplate::contract_call("0x1234567890123456789012345678901234567890", vec![0x42, 0x84, 0x2e, 0x0e]), // transfer()
            ],
            description: "NFT transactions: 70% transfers, 20% minting, 10% marketplace".to_string(),
        }
    }

    /// Select a transaction template based on weights
    pub fn select_template(&self) -> &TransactionTemplate {
        let total_weight: f64 = self.templates.iter().map(|t| t.weight).sum();
        let mut rng = rand::thread_rng();
        let mut random_weight = rng.gen::<f64>() * total_weight;

        for template in &self.templates {
            random_weight -= template.weight;
            if random_weight <= 0.0 {
                return template;
            }
        }

        // Fallback to first template
        &self.templates[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_template_creation() {
        let template = TransactionTemplate::transfer(1000000000000000000);
        assert_eq!(template.name, "Simple Transfer");
        assert_eq!(template.gas_limit, 21000);
        assert_eq!(template.value, 1000000000000000000);
        assert!(template.data.is_none());
        assert!(template.to.is_none());
    }

    #[test]
    fn test_erc20_template_creation() {
        let template = TransactionTemplate::erc20_transfer("0x1234567890123456789012345678901234567890", 1000000);
        assert_eq!(template.name, "ERC-20 Transfer");
        assert_eq!(template.gas_limit, 60000);
        assert_eq!(template.value, 0);
        assert!(template.data.is_some());
        assert_eq!(template.to, Some("0x1234567890123456789012345678901234567890".to_string()));
    }

    #[test]
    fn test_workload_pattern_selection() {
        let pattern = WorkloadPattern::simple_transfers();
        assert_eq!(pattern.name, "Simple Transfers");
        assert_eq!(pattern.templates.len(), 1);

        let selected = pattern.select_template();
        assert_eq!(selected.name, "Simple Transfer");
    }

    #[test]
    fn test_test_contracts() {
        let counter = TestContracts::counter_contract();
        assert!(!counter.is_empty());

        let erc20 = TestContracts::erc20_contract();
        assert!(!erc20.is_empty());

        let empty = TestContracts::empty_contract();
        assert!(!empty.is_empty());
    }

    #[test]
    fn test_defi_workload() {
        let pattern = WorkloadPattern::defi_workload();
        assert_eq!(pattern.name, "DeFi Workload");
        assert_eq!(pattern.templates.len(), 3);

        // Test selection multiple times to ensure randomness
        let mut selected_types = std::collections::HashSet::new();
        for _ in 0..100 {
            let template = pattern.select_template();
            selected_types.insert(template.name.clone());
        }

        // Should have selected from multiple templates
        assert!(selected_types.len() > 1);
    }
}