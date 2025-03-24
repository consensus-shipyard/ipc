// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Error;
use fendermint_actor_blobs_shared::params::{
    ApproveCreditParams, BuyCreditParams, GetAccountParams, GetCreditApprovalParams,
    RevokeCreditParams, SetAccountStatusParams, SetSponsorParams,
};
use fendermint_actor_blobs_shared::state::{Credit, CreditApproval, TtlStatus};
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{actor_error, ActorError};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use recall_actor_sdk::{token_to_biguint, TryIntoEVMEvent};
use recall_sol_facade::credit as sol;
use recall_sol_facade::primitives::U256;
use recall_sol_facade::types::{BigUintWrapper, SolCall, SolInterface, H160};
use std::collections::{HashMap, HashSet};

pub use recall_sol_facade::credit::Calls;

use crate::sol_facade::{AbiCall, AbiCallRuntime, AbiEncodeError};
use crate::state::AccountInfo;

pub struct CreditPurchased {
    from: Address,
    amount: TokenAmount,
}
impl CreditPurchased {
    pub fn new(from: Address, amount: TokenAmount) -> Self {
        Self { from, amount }
    }
}
impl TryIntoEVMEvent for CreditPurchased {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        let from: H160 = self.from.try_into()?;
        let amount = token_to_biguint(Some(self.amount));
        Ok(sol::Events::CreditPurchased(sol::CreditPurchased {
            from: from.into(),
            amount: BigUintWrapper(amount).into(),
        }))
    }
}

pub struct CreditApproved {
    pub from: Address,
    pub to: Address,
    pub credit_limit: Option<TokenAmount>,
    pub gas_fee_limit: Option<TokenAmount>,
    pub expiry: Option<ChainEpoch>,
}
impl TryIntoEVMEvent for CreditApproved {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<sol::Events, Error> {
        let from: H160 = self.from.try_into()?;
        let to: H160 = self.to.try_into()?;
        let credit_limit = token_to_biguint(self.credit_limit);
        let gas_fee_limit = token_to_biguint(self.gas_fee_limit);
        Ok(sol::Events::CreditApproved(sol::CreditApproved {
            from: from.into(),
            to: to.into(),
            creditLimit: BigUintWrapper(credit_limit).into(),
            gasFeeLimit: BigUintWrapper(gas_fee_limit).into(),
            expiry: U256::from(self.expiry.unwrap_or_default()),
        }))
    }
}

pub struct CreditRevoked {
    pub from: Address,
    pub to: Address,
}
impl CreditRevoked {
    pub fn new(from: Address, to: Address) -> Self {
        Self { from, to }
    }
}
impl TryIntoEVMEvent for CreditRevoked {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<sol::Events, Error> {
        let from: H160 = self.from.try_into()?;
        let to: H160 = self.to.try_into()?;
        Ok(sol::Events::CreditRevoked(sol::CreditRevoked {
            from: from.into(),
            to: to.into(),
        }))
    }
}

pub struct CreditDebited {
    pub amount: TokenAmount,
    pub num_accounts: u64,
    pub more_accounts: bool,
}
impl TryIntoEVMEvent for CreditDebited {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<sol::Events, Error> {
        let amount = token_to_biguint(Some(self.amount));
        Ok(sol::Events::CreditDebited(sol::CreditDebited {
            amount: BigUintWrapper(amount).into(),
            numAccounts: U256::from(self.num_accounts),
            moreAccounts: self.more_accounts,
        }))
    }
}

// ----- Calls ----- //

pub fn can_handle(input_data: &recall_actor_sdk::InputData) -> bool {
    Calls::valid_selector(input_data.selector())
}

pub fn parse_input(input: &recall_actor_sdk::InputData) -> Result<Calls, ActorError> {
    Calls::abi_decode_raw(input.selector(), input.calldata(), true)
        .map_err(|e| actor_error!(illegal_argument, format!("invalid call: {}", e)))
}

/// function buyCredit() external payable;
impl AbiCallRuntime for sol::buyCredit_0Call {
    type Params = BuyCreditParams;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let recipient = rt.message().caller();
        BuyCreditParams(recipient)
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

/// function buyCredit(address recipient) external payable;
impl AbiCall for sol::buyCredit_1Call {
    type Params = BuyCreditParams;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let recipient: Address = H160::from(self.recipient).into();
        BuyCreditParams(recipient)
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

/// function approveCredit(address to) external;
impl AbiCallRuntime for sol::approveCredit_0Call {
    type Params = ApproveCreditParams;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let from = rt.message().caller();
        let to: Address = H160::from(self.to).into();
        ApproveCreditParams {
            from,
            to,
            caller_allowlist: None,
            credit_limit: None,
            gas_fee_limit: None,
            ttl: None,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

/// function approveCredit(address to, address[] memory caller, uint256 creditLimit, uint256 gasFeeLimit, uint64 ttl) external;
impl AbiCallRuntime for sol::approveCredit_1Call {
    type Params = ApproveCreditParams;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let from: Address = rt.message().caller();
        let to: Address = H160::from(self.to).into();
        let caller_allowlist: HashSet<Address> = HashSet::from_iter(
            self.caller
                .iter()
                .map(|sol_address| H160::from(*sol_address).into()),
        );
        let credit_limit: Credit = BigUintWrapper::from(self.creditLimit).into();
        let gas_fee_limit: TokenAmount = BigUintWrapper::from(self.gasFeeLimit).into();
        let ttl = self.ttl;
        ApproveCreditParams {
            from,
            to,
            caller_allowlist: Some(caller_allowlist),
            credit_limit: Some(credit_limit),
            gas_fee_limit: Some(gas_fee_limit),
            ttl: Some(ttl as ChainEpoch),
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

/// function approveCredit(address to, address[] memory caller) external;
impl AbiCallRuntime for sol::approveCredit_2Call {
    type Params = ApproveCreditParams;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let from: Address = rt.message().caller();
        let to: Address = H160::from(self.to).into();
        let caller_allowlist: HashSet<Address> = HashSet::from_iter(
            self.caller
                .iter()
                .map(|sol_address| H160::from(*sol_address).into()),
        );
        ApproveCreditParams {
            from,
            to,
            caller_allowlist: Some(caller_allowlist),
            credit_limit: None,
            gas_fee_limit: None,
            ttl: None,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

/// function revokeCredit(address to, address caller) external;
impl AbiCallRuntime for sol::revokeCredit_0Call {
    type Params = RevokeCreditParams;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let from: Address = rt.message().caller();
        let to: Address = H160::from(self.to).into();
        let caller: Address = H160::from(self.caller).into();
        RevokeCreditParams {
            from,
            to,
            for_caller: Some(caller),
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

/// function revokeCredit(address to) external;
impl AbiCallRuntime for sol::revokeCredit_1Call {
    type Params = RevokeCreditParams;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let from: Address = rt.message().caller();
        let to: Address = H160::from(self.to).into();
        RevokeCreditParams {
            from,
            to,
            for_caller: None,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

/// function setAccountSponsor(address from, address sponsor) external;
impl AbiCallRuntime for sol::setAccountSponsorCall {
    type Params = SetSponsorParams; // FIXME SU Needs runtime for "from"
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let from: Address = rt.message().caller();
        let sponsor = H160::from(self.sponsor);
        let sponsor: Option<Address> = if sponsor.is_null() {
            None
        } else {
            Some(sponsor.into())
        };
        SetSponsorParams { from, sponsor }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

fn convert_approvals(
    approvals: HashMap<Address, CreditApproval>,
) -> Result<Vec<sol::Approval>, anyhow::Error> {
    approvals
        .iter()
        .map(|(address, credit_approval)| {
            let approval = sol::Approval {
                addr: H160::try_from(*address)?.into(),
                approval: sol::CreditApproval {
                    creditLimit: credit_approval
                        .credit_limit
                        .clone()
                        .map(BigUintWrapper::from)
                        .unwrap_or_default()
                        .into(),
                    gasFeeLimit: credit_approval
                        .gas_fee_limit
                        .clone()
                        .map(BigUintWrapper::from)
                        .unwrap_or_default()
                        .into(),
                    expiry: credit_approval.expiry.unwrap_or_default() as u64,
                    creditUsed: BigUintWrapper::from(credit_approval.credit_used.clone()).into(),
                    gasFeeUsed: BigUintWrapper::from(credit_approval.gas_fee_used.clone()).into(),
                },
            };
            Ok(approval)
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()
}

/// function getAccount(address addr) external view returns (Account memory account);
impl AbiCall for sol::getAccountCall {
    type Params = GetAccountParams;
    type Returns = Option<AccountInfo>;
    type Output = Result<Vec<u8>, AbiEncodeError>;

    fn params(&self) -> Self::Params {
        let address: Address = H160::from(self.addr).into();
        GetAccountParams(address)
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let sol_account = if let Some(account) = returns {
            let credit_sponsor: H160 = account
                .credit_sponsor
                .map(H160::try_from)
                .transpose()?
                .unwrap_or_default();
            let approvals_from = convert_approvals(account.approvals_from)?;
            let approvals_to = convert_approvals(account.approvals_to)?;
            sol::Account {
                capacityUsed: account.capacity_used,
                creditFree: BigUintWrapper::from(account.credit_free).into(),
                creditCommitted: BigUintWrapper::from(account.credit_committed).into(),
                creditSponsor: credit_sponsor.into(),
                lastDebitEpoch: account.last_debit_epoch as u64,
                approvalsFrom: approvals_from,
                approvalsTo: approvals_to,
                maxTtl: account.max_ttl as u64,
                gasAllowance: BigUintWrapper::from(account.gas_allowance).into(),
            }
        } else {
            sol::Account {
                capacityUsed: u64::default(),
                creditFree: U256::default(),
                creditCommitted: U256::default(),
                creditSponsor: H160::default().into(),
                lastDebitEpoch: u64::default(),
                approvalsTo: Vec::default(),
                approvalsFrom: Vec::default(),
                maxTtl: u64::default(),
                gasAllowance: U256::default(),
            }
        };
        Ok(Self::abi_encode_returns(&(sol_account,)))
    }
}

/// function getCreditApproval(address from, address to) external view returns (CreditApproval memory approval);
impl AbiCall for sol::getCreditApprovalCall {
    type Params = GetCreditApprovalParams;
    type Returns = Option<CreditApproval>;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let from = H160::from(self.from);
        let to = H160::from(self.to);
        GetCreditApprovalParams {
            from: from.into(),
            to: to.into(),
        }
    }

    fn returns(&self, value: Self::Returns) -> Self::Output {
        let approval_result = if let Some(credit_approval) = value {
            sol::CreditApproval {
                creditLimit: credit_approval
                    .credit_limit
                    .clone()
                    .map(BigUintWrapper::from)
                    .unwrap_or_default()
                    .into(),
                gasFeeLimit: credit_approval
                    .gas_fee_limit
                    .clone()
                    .map(BigUintWrapper::from)
                    .unwrap_or_default()
                    .into(),
                expiry: credit_approval.expiry.unwrap_or_default() as u64,
                creditUsed: BigUintWrapper::from(credit_approval.credit_used.clone()).into(),
                gasFeeUsed: BigUintWrapper::from(credit_approval.gas_fee_used.clone()).into(),
            }
        } else {
            sol::CreditApproval {
                creditLimit: BigUintWrapper::default().into(),
                gasFeeLimit: BigUintWrapper::default().into(),
                expiry: u64::default(),
                creditUsed: BigUintWrapper::default().into(),
                gasFeeUsed: BigUintWrapper::default().into(),
            }
        };
        Self::abi_encode_returns(&(approval_result,))
    }
}

/// function setAccountStatus(address subscriber, TtlStatus ttlStatus) external;
impl AbiCall for sol::setAccountStatusCall {
    type Params = Result<SetAccountStatusParams, ActorError>;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let subscriber = H160::from(self.subscriber);
        let ttl_status = match self.ttlStatus {
            0 => TtlStatus::Default,
            1 => TtlStatus::Reduced,
            2 => TtlStatus::Extended,
            _ => return Err(actor_error!(illegal_argument, "invalid TtlStatus")),
        };
        Ok(SetAccountStatusParams {
            subscriber: subscriber.into(),
            status: ttl_status,
        })
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}
