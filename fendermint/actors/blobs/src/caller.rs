// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::credit::{
    Credit, CreditAllowance, CreditApproval, GasAllowance,
};
use fendermint_actor_recall_config_shared::RecallConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use log::debug;
use num_traits::Zero;
use recall_ipld::hamt;

use crate::state::accounts::Account;

/// Helper for managing blobs actor state caller.
#[allow(clippy::large_enum_variant)]
pub enum Caller<'a, BS: Blockstore> {
    Default((Address, Account)),
    Sponsored(Delegation<'a, &'a BS>),
}

impl<'a, BS: Blockstore> Caller<'a, BS> {
    /// Loads the caller and optional sponsor account with its delegation.
    pub fn load(
        store: &'a BS,
        accounts: &hamt::map::Hamt<'a, &'a BS, Address, Account>,
        caller: Address,
        sponsor: Option<Address>,
    ) -> Result<Self, ActorError> {
        let account = accounts.get_or_err(&caller)?;
        Self::load_account(store, accounts, caller, account, sponsor)
    }

    /// Loads the caller and the caller's default sponsor with its delegation.
    /// If the sponsor does not exist or the caller does not have an approval from
    /// the default sponsor, a default caller type is returned.
    pub fn load_with_default_sponsor(
        store: &'a BS,
        accounts: &hamt::map::Hamt<'a, &'a BS, Address, Account>,
        caller: Address,
    ) -> Result<Self, ActorError> {
        let account = accounts.get_or_err(&caller)?;
        match Self::load_account(
            store,
            accounts,
            caller,
            account.clone(),
            account.credit_sponsor,
        ) {
            Ok(caller) => Ok(caller),
            Err(_) => Self::load_account(store, accounts, caller, account, None),
        }
    }

    /// Loads the caller and optional sponsor account with its delegation.
    /// The caller account will be created if one does not exist.
    pub fn load_or_create(
        store: &'a BS,
        accounts: &hamt::map::Hamt<'a, &'a BS, Address, Account>,
        caller: Address,
        sponsor: Option<Address>,
        current_epoch: ChainEpoch,
        max_ttl: ChainEpoch,
    ) -> Result<Self, ActorError> {
        let account =
            accounts.get_or_create(&caller, || Account::new(store, current_epoch, max_ttl))?;
        Self::load_account(store, accounts, caller, account, sponsor)
    }

    /// Loads the caller and optional sponsor account with its delegation.
    pub fn load_account(
        store: &'a BS,
        accounts: &hamt::map::Hamt<'a, &'a BS, Address, Account>,
        caller: Address,
        caller_account: Account,
        sponsor: Option<Address>,
    ) -> Result<Self, ActorError> {
        let sponsor = sponsor.unwrap_or(caller);
        if sponsor != caller {
            let delegation = Delegation::load(store, accounts, sponsor, caller, caller_account)?;
            Ok(Self::Sponsored(delegation))
        } else {
            Ok(Self::Default((caller, caller_account)))
        }
    }

    /// Returns the caller address.
    #[allow(dead_code)]
    pub fn address(&self) -> Address {
        match self {
            Self::Default((address, _)) => *address,
            Self::Sponsored(delegation) => delegation.to,
        }
    }

    /// Returns the subscriber address.
    /// The subscriber is the account responsible for credit and gas fees.
    /// The subscriber is the caller or the sponsor if one exists.
    pub fn subscriber_address(&self) -> Address {
        match self {
            Self::Default((address, _)) => *address,
            Self::Sponsored(delegation) => delegation.from,
        }
    }

    /// Returns the delegate address.
    /// The delegate only exists if there's a sponsor.
    /// If present, the delegate address will be the caller address.
    pub fn delegate_address(&self) -> Option<Address> {
        match self {
            Self::Default(_) => None,
            Self::Sponsored(delegation) => Some(delegation.to),
        }
    }

    /// Returns the underlying delegate approval.
    /// The delegate only exists if there's a sponsor.
    pub fn delegate_approval(&self) -> Option<&CreditApproval> {
        match self {
            Self::Default(_) => None,
            Self::Sponsored(delegation) => Some(&delegation.approval_to),
        }
    }

    /// Returns the subscriber account.
    /// The subscriber is the account responsible for credit and gas fees.
    /// The subscriber is the caller or the sponsor if one exists.
    pub fn subscriber(&self) -> &Account {
        match self {
            Self::Default((_, account)) => account,
            Self::Sponsored(delegation) => &delegation.from_account,
        }
    }

    /// Returns the subscriber account as a mutable reference.
    /// The subscriber is the account responsible for credit and gas fees.
    /// The subscriber is the caller or the sponsor if one exists.
    #[allow(dead_code)]
    pub fn subscriber_mut(&mut self) -> &mut Account {
        match self {
            Self::Default((_, account)) => account,
            Self::Sponsored(delegation) => &mut delegation.from_account,
        }
    }

    /// Returns whether the caller is a delegate.
    pub fn is_delegate(&self) -> bool {
        matches!(self, Self::Sponsored(_))
    }

    /// Sets the default sponsor for the caller or the delegate.
    pub fn set_default_sponsor(&mut self, sponsor: Option<Address>) {
        match self {
            Self::Default((_, account)) => account.credit_sponsor = sponsor,
            Self::Sponsored(delegation) => {
                delegation.to_account.credit_sponsor = sponsor;
            }
        }
    }

    /// Adds credit and gas allowances to the subscriber.
    pub fn add_allowances(&mut self, credit: &Credit, value: &TokenAmount) {
        match self {
            Self::Default((_, account)) => {
                account.credit_free += credit;
                account.gas_allowance += value;
            }
            Self::Sponsored(delegation) => {
                delegation.from_account.credit_free += credit;
                delegation.from_account.gas_allowance += value;
            }
        }

        debug!("added {} credits to {}", credit, self.subscriber_address());
        debug!(
            "added {} gas fee allowance to {}",
            value,
            self.subscriber_address()
        );
    }

    /// Returns the credit allowance for the subscriber.
    #[allow(dead_code)]
    pub fn credit_allowance(&self, current_epoch: ChainEpoch) -> CreditAllowance {
        match self {
            Self::Default((_, account)) => CreditAllowance {
                amount: account.credit_free.clone(),
                ..Default::default()
            },
            Self::Sponsored(delegation) => delegation.credit_allowance(current_epoch),
        }
    }

    /// Returns the gas allowance for the subscriber.
    pub fn gas_allowance(&self, current_epoch: ChainEpoch) -> GasAllowance {
        match self {
            Self::Default((_, account)) => GasAllowance {
                amount: account.gas_allowance.clone(),
                ..Default::default()
            },
            Self::Sponsored(delegation) => delegation.gas_allowance(current_epoch),
        }
    }

    /// Commits new capacity for the subscriber.
    pub fn commit_capacity(
        &mut self,
        size: u64,
        cost: &Credit,
        current_epoch: ChainEpoch,
    ) -> Result<(), ActorError> {
        // Check subscriber's free credit
        if &self.subscriber().credit_free < cost {
            return Err(ActorError::insufficient_funds(format!(
                "account {} has insufficient credit (available: {}; required: {})",
                self.subscriber_address(),
                &self.subscriber().credit_free,
                cost
            )));
        }
        match self {
            Self::Default((_, account)) => {
                account.capacity_used += size;
                account.credit_free -= cost;
                account.credit_committed += cost;
            }
            Self::Sponsored(delegation) => {
                delegation.use_credit_allowance(cost, current_epoch)?;
                delegation.from_account.capacity_used += size;
                delegation.from_account.credit_free -= cost;
                delegation.from_account.credit_committed += cost;
            }
        }

        debug!("used {} bytes from {}", size, self.subscriber_address());
        debug!(
            "committed {} credits from {}",
            cost,
            self.subscriber_address()
        );

        Ok(())
    }

    /// Releases capacity for the subscriber.
    pub fn release_capacity(&mut self, size: u64, cost: &Credit) {
        match self {
            Self::Default((_, account)) => {
                account.capacity_used -= size;
                account.credit_free += cost;
                account.credit_committed -= cost;
            }
            Self::Sponsored(delegation) => {
                delegation.return_credit_allowance(cost);
                delegation.from_account.capacity_used -= size;
                delegation.from_account.credit_free += cost;
                delegation.from_account.credit_committed -= cost;
            }
        }

        debug!("released {} bytes to {}", size, self.subscriber_address());
        debug!("released {} credits to {}", cost, self.subscriber_address());
    }

    /// Debit credits from the subscriber.
    pub fn debit_credit(&mut self, amount: &Credit, current_epoch: ChainEpoch) {
        match self {
            Self::Default((_, account)) => {
                account.credit_committed -= amount;
                account.last_debit_epoch = current_epoch;
            }
            Self::Sponsored(delegation) => {
                delegation.from_account.credit_committed -= amount;
                delegation.from_account.last_debit_epoch = current_epoch;
            }
        }

        debug!(
            "debited {} credits from {}",
            amount,
            self.subscriber_address()
        );
    }

    /// Refund credit to the subscriber.
    pub fn refund_credit(&mut self, amount: &Credit, correction: &Credit) {
        match self {
            Self::Default((_, account)) => {
                account.credit_free += amount - correction;
                account.credit_committed += correction;
            }
            Self::Sponsored(delegation) => {
                delegation.from_account.credit_free += amount - correction;
                delegation.from_account.credit_committed += correction;
            }
        }

        debug!(
            "refunded {} credits to {}",
            amount - correction,
            self.subscriber_address()
        );
    }

    /// Returns committed credits to the subscriber.
    pub fn return_committed_credit(&mut self, amount: &Credit) {
        match self {
            Self::Default((_, account)) => {
                account.credit_committed += amount;
            }
            Self::Sponsored(delegation) => {
                delegation.from_account.credit_committed += amount;
            }
        }

        debug!(
            "returned {} committed credits to {}",
            amount,
            self.subscriber_address()
        );
    }

    /// Updates gas allowance for the subscriber.
    pub fn update_gas_allowance(
        &mut self,
        add_amount: &TokenAmount,
        current_epoch: ChainEpoch,
    ) -> Result<(), ActorError> {
        match self {
            Self::Default((_, account)) => {
                account.gas_allowance += add_amount;
            }
            Self::Sponsored(delegation) => {
                if add_amount.is_positive() {
                    delegation.return_gas_allowance(add_amount);
                } else if add_amount.is_negative() {
                    delegation.use_gas_allowance(&-add_amount, current_epoch)?;
                }
                delegation.from_account.gas_allowance += add_amount;
            }
        }

        if add_amount.is_positive() {
            debug!(
                "refunded {} atto to {}",
                add_amount.atto(),
                self.subscriber_address()
            );
        } else {
            debug!(
                "debited {} atto from {}",
                -add_amount.atto(),
                self.subscriber_address()
            );
        }
        Ok(())
    }

    /// Validates the delegate expiration.
    pub fn validate_delegate_expiration(
        &self,
        current_epoch: ChainEpoch,
    ) -> Result<(), ActorError> {
        match self {
            Self::Default(_) => Ok(()),
            Self::Sponsored(delegation) => delegation.validate_expiration(current_epoch),
        }
    }

    /// Validates a blob TTL for the subscriber.
    pub fn validate_ttl_usage(
        &self,
        config: &RecallConfig,
        ttl: Option<ChainEpoch>,
    ) -> Result<ChainEpoch, ActorError> {
        let ttl = ttl.unwrap_or(config.blob_default_ttl);
        if ttl < config.blob_min_ttl {
            return Err(ActorError::illegal_argument(format!(
                "minimum blob TTL is {}",
                config.blob_min_ttl
            )));
        } else if ttl > self.subscriber().max_ttl {
            return Err(ActorError::forbidden(format!(
                "attempt to add a blob with TTL ({}) that exceeds account's max allowed TTL ({})",
                ttl,
                self.subscriber().max_ttl,
            )));
        }
        Ok(ttl)
    }

    /// Saves state to accounts.
    pub fn save(
        &mut self,
        accounts: &mut hamt::map::Hamt<'a, &'a BS, Address, Account>,
    ) -> Result<(), ActorError> {
        match self {
            Self::Default((address, account)) => {
                accounts.set(address, account.clone())?;
                Ok(())
            }
            Self::Sponsored(delegation) => delegation.save(accounts),
        }
    }

    /// Cancels the optional delegation and converts to the default caller type.
    pub fn cancel_delegation(
        &mut self,
        accounts: &mut hamt::map::Hamt<'a, &'a BS, Address, Account>,
    ) -> Result<(), ActorError> {
        match self {
            Self::Default(_) => Ok(()),
            Self::Sponsored(delegation) => {
                delegation.cancel(accounts)?;
                // Delegation is now invalid, convert to default caller type
                *self = Self::Default((delegation.to, delegation.to_account.clone()));
                Ok(())
            }
        }
    }
}

/// Helper for handling credit approvals.
pub struct Delegation<'a, BS: Blockstore> {
    /// The issuer address.
    from: Address,
    /// The issuer account.
    from_account: Account,
    /// The recipient address.
    to: Address,
    /// The recipient account.
    to_account: Account,
    /// Approvals from issuer to recipient.
    approvals_from: hamt::map::Hamt<'a, BS, Address, CreditApproval>,
    /// Approvals to recipient from issuer.
    approvals_to: hamt::map::Hamt<'a, BS, Address, CreditApproval>,
    /// Approval from issuer to recipient.
    approval_from: CreditApproval,
    /// Approval to recipient from issuer.
    approval_to: CreditApproval,
}

/// Options for creating a new delegation.
#[derive(Debug, Default)]
pub struct DelegationOptions {
    /// Optional credit limit.
    pub credit_limit: Option<Credit>,
    /// Optional gas fee limit.
    pub gas_fee_limit: Option<TokenAmount>,
    /// Optional time-to-live (TTL).
    pub ttl: Option<ChainEpoch>,
}

impl<'a, BS: Blockstore> Delegation<'a, &'a BS> {
    /// Loads an existing delegation.
    pub fn load(
        store: &'a BS,
        accounts: &hamt::map::Hamt<'a, &'a BS, Address, Account>,
        from: Address,
        to: Address,
        to_account: Account,
    ) -> Result<Self, ActorError> {
        if from == to {
            return Err(ActorError::illegal_argument(
                "'from' and 'to' addresses must be different".into(),
            ));
        }

        let from_account = accounts.get_or_err(&from)?;
        let approvals_to = from_account.approvals_to.hamt(store)?;
        let approval_to = approvals_to.get(&to)?.ok_or(ActorError::forbidden(format!(
            "approval to {} from {} not found",
            to, from
        )))?;
        let approvals_from = to_account.approvals_from.hamt(store)?;
        let approval_from = approvals_from
            .get(&from)?
            .ok_or(ActorError::forbidden(format!(
                "approval from {} to {} not found",
                from, to
            )))?;

        Ok(Self {
            from,
            from_account,
            to,
            to_account,
            approvals_from,
            approvals_to,
            approval_from,
            approval_to,
        })
    }

    /// Creates a new delegation from one account to another.
    pub fn update_or_create(
        store: &'a BS,
        config: &RecallConfig,
        accounts: &hamt::map::Hamt<'a, &'a BS, Address, Account>,
        from: Address,
        to: Address,
        options: DelegationOptions,
        current_epoch: ChainEpoch,
    ) -> Result<Self, ActorError> {
        if let Some(ttl) = options.ttl {
            if ttl < config.blob_min_ttl {
                return Err(ActorError::illegal_argument(format!(
                    "minimum approval TTL is {}",
                    config.blob_min_ttl
                )));
            }
        }

        let expiry = options.ttl.map(|t| i64::saturating_add(t, current_epoch));
        let approval = CreditApproval {
            credit_limit: options.credit_limit.clone(),
            gas_allowance_limit: options.gas_fee_limit.clone(),
            expiry,
            credit_used: Credit::zero(),
            gas_allowance_used: TokenAmount::zero(),
        };

        // Get or create accounts
        let from_account = accounts.get_or_create(&from, || {
            Account::new(store, current_epoch, config.blob_default_ttl)
        })?;
        let to_account = accounts.get_or_create(&to, || {
            Account::new(store, current_epoch, config.blob_default_ttl)
        })?;

        // Get or create approvals
        let approvals_to = from_account.approvals_to.hamt(store)?;
        let approvals_from = to_account.approvals_from.hamt(store)?;
        let mut approval_to = approvals_to.get_or_create(&to, || Ok(approval.clone()))?;
        let mut approval_from = approvals_from.get_or_create(&from, || Ok(approval))?;
        if approval_from != approval_to {
            return Err(ActorError::illegal_state(format!(
                "'from' account ({}) approval does not match 'to' account ({}) approval",
                from, to,
            )));
        }

        // Validate approval changes (check one of them since they are equal)
        if let Some(limit) = options.credit_limit.as_ref() {
            if &approval_to.credit_used > limit {
                return Err(ActorError::illegal_argument(format!(
                    "limit cannot be less than amount of already used credits ({})",
                    approval_to.credit_used
                )));
            }
        }
        if let Some(limit) = options.gas_fee_limit.as_ref() {
            if &approval_to.gas_allowance_used > limit {
                return Err(ActorError::illegal_argument(format!(
                    "limit cannot be less than amount of already used gas fees ({})",
                    approval_to.gas_allowance_used
                )));
            }
        }

        approval_from.credit_limit = options.credit_limit.clone();
        approval_from.gas_allowance_limit = options.gas_fee_limit.clone();
        approval_from.expiry = expiry;
        approval_to.credit_limit = options.credit_limit;
        approval_to.gas_allowance_limit = options.gas_fee_limit;
        approval_to.expiry = expiry;

        debug!(
            "approval created from {} to {} (credit limit: {:?}; gas fee limit: {:?}, expiry: {:?}",
            from,
            to,
            approval_from.credit_limit,
            approval_from.gas_allowance_limit,
            approval_from.expiry
        );

        Ok(Self {
            to,
            to_account,
            from,
            from_account,
            approvals_from,
            approvals_to,
            approval_from,
            approval_to,
        })
    }

    /// Return credit allowance to the delegation.
    pub fn return_credit_allowance(&mut self, amount: &Credit) {
        self.approval_from.credit_used -= amount;
        self.approval_to.credit_used -= amount;
    }

    /// Use credit allowance from the delegation.
    pub fn use_credit_allowance(
        &mut self,
        amount: &Credit,
        current_epoch: ChainEpoch,
    ) -> Result<(), ActorError> {
        self.validate_expiration(current_epoch)?;
        self.validate_credit_usage(amount)?;
        self.approval_from.credit_used += amount;
        self.approval_to.credit_used += amount;
        Ok(())
    }

    /// Return gas allowance to the delegation.
    pub fn return_gas_allowance(&mut self, amount: &TokenAmount) {
        self.approval_from.gas_allowance_used -= amount;
        self.approval_to.gas_allowance_used -= amount;
    }

    /// Use gas allowance from the delegation.
    pub fn use_gas_allowance(
        &mut self,
        amount: &TokenAmount,
        current_epoch: ChainEpoch,
    ) -> Result<(), ActorError> {
        self.validate_expiration(current_epoch)?;
        self.validate_gas_usage(amount)?;
        self.approval_from.gas_allowance_used += amount;
        self.approval_to.gas_allowance_used += amount;
        Ok(())
    }

    /// Saves state to accounts.
    pub fn save(
        &mut self,
        accounts: &mut hamt::map::Hamt<'a, &'a BS, Address, Account>,
    ) -> Result<(), ActorError> {
        // Save the "from" account's "to" approval
        self.from_account.approvals_to.save_tracked(
            self.approvals_to
                .set_and_flush_tracked(&self.to, self.approval_to.clone())?,
        );
        // Save the "to" account's "from" approval
        self.to_account.approvals_from.save_tracked(
            self.approvals_from
                .set_and_flush_tracked(&self.from, self.approval_from.clone())?,
        );
        // Save the "from" account
        accounts.set(&self.from, self.from_account.clone())?;
        // Save the "to" account
        accounts.set(&self.to, self.to_account.clone())?;
        Ok(())
    }

    /// Cancels the underlying approval and saves state to accounts.
    pub fn cancel(
        &mut self,
        accounts: &mut hamt::map::Hamt<'a, &'a BS, Address, Account>,
    ) -> Result<(), ActorError> {
        // Remove the "from" account's "to" approval
        self.from_account
            .approvals_to
            .save_tracked(self.approvals_to.delete_and_flush_tracked(&self.to)?.0);
        // Remove the "to" account's "from" approval
        self.to_account
            .approvals_from
            .save_tracked(self.approvals_from.delete_and_flush_tracked(&self.from)?.0);
        // Save the "from" account
        accounts.set(&self.from, self.from_account.clone())?;
        // Save the "to" account
        accounts.set(&self.to, self.to_account.clone())?;

        debug!("approval canceled from {} to {}", self.from, self.to);
        Ok(())
    }

    /// Returns the underlying approval.
    pub fn approval(&self) -> &CreditApproval {
        &self.approval_to
    }

    /// Returns the credit allowance for the subscriber.
    #[allow(dead_code)]
    pub fn credit_allowance(&self, current_epoch: ChainEpoch) -> CreditAllowance {
        let mut allowance = CreditAllowance {
            amount: self.to_account.credit_free.clone(),
            sponsor: Some(self.from),
            sponsored_amount: Credit::zero(),
        };
        if self.validate_expiration(current_epoch).is_err() {
            return allowance;
        }
        let approval_used = self.approval_to.credit_used.clone();
        let approval_allowance = self.from_account.credit_free.clone();
        let approval_allowance = self
            .approval_to
            .credit_limit
            .clone()
            .map_or(approval_allowance.clone(), |limit| {
                (limit - approval_used).min(approval_allowance)
            });
        allowance.sponsored_amount = approval_allowance;
        allowance
    }

    /// Returns the gas allowance for the subscriber.
    pub fn gas_allowance(&self, current_epoch: ChainEpoch) -> GasAllowance {
        let mut allowance = GasAllowance {
            amount: self.to_account.gas_allowance.clone(),
            sponsor: Some(self.from),
            sponsored_amount: TokenAmount::zero(),
        };
        if self.validate_expiration(current_epoch).is_err() {
            return allowance;
        }
        let approval_used = self.approval_to.gas_allowance_used.clone();
        let approval_allowance = self.from_account.gas_allowance.clone();
        let approval_allowance = self
            .approval_to
            .gas_allowance_limit
            .clone()
            .map_or(approval_allowance.clone(), |limit| {
                (limit - approval_used).min(approval_allowance)
            });
        allowance.sponsored_amount = approval_allowance;
        allowance
    }

    /// Validates whether the delegation has valid expiry for the epoch.
    pub fn validate_expiration(&self, current_epoch: ChainEpoch) -> Result<(), ActorError> {
        self.approval_from.validate_expiration(current_epoch)?;
        self.approval_to.validate_expiration(current_epoch)?;
        Ok(())
    }

    /// Validates whether the delegation can use the amount of credit.
    pub fn validate_credit_usage(&self, amount: &Credit) -> Result<(), ActorError> {
        self.approval_from.validate_credit_usage(amount)?;
        self.approval_to.validate_credit_usage(amount)?;
        Ok(())
    }

    /// Validates whether the delegation can use the amount of gas.
    pub fn validate_gas_usage(&self, amount: &TokenAmount) -> Result<(), ActorError> {
        self.approval_from.validate_gas_usage(amount)?;
        self.approval_to.validate_gas_usage(amount)?;
        Ok(())
    }
}
