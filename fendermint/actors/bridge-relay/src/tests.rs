// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Unit tests for the bridge-relay actor (shared logic only; no FVM runtime needed).

#[cfg(test)]
mod tests {
    use fvm_ipld_blockstore::MemoryBlockstore;
    use fvm_shared::address::Address;
    use fvm_shared::econ::TokenAmount;

    use crate::{
        State, TokensLockedEvent, TransferId, ValidationError, ValidationRules,
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_addr(id: u64) -> Address {
        Address::new_id(id)
    }

    fn make_transfer_id(n: u8) -> TransferId {
        let mut id = [0u8; 32];
        id[31] = n;
        id
    }

    fn default_rules() -> ValidationRules {
        ValidationRules {
            min_amount: TokenAmount::from_atto(0u64),
            max_amount: TokenAmount::from_atto(0u64),
            allowed_tokens: vec![],
        }
    }

    fn make_event(token: Address, recipient: Address, amount: u128) -> TokensLockedEvent {
        TokensLockedEvent {
            token,
            sender: make_addr(999),
            recipient,
            amount: TokenAmount::from_atto(amount),
            transfer_id: make_transfer_id(1),
        }
    }

    fn make_state() -> (State, MemoryBlockstore) {
        let store = MemoryBlockstore::default();
        let state = State::new(
            &store,
            make_addr(1),
            make_addr(2),
            default_rules(),
        )
        .expect("state creation failed");
        (state, store)
    }

    // ── ValidationRules::validate ─────────────────────────────────────────────

    #[test]
    fn test_validate_accepts_valid_event() {
        let rules = default_rules();
        let event = make_event(make_addr(10), make_addr(20), 100);
        assert!(rules.validate(&event).is_ok());
    }

    #[test]
    fn test_validate_rejects_zero_amount() {
        let rules = default_rules();
        let mut event = make_event(make_addr(10), make_addr(20), 0);
        event.amount = TokenAmount::from_atto(0u64);
        let err = rules.validate(&event).unwrap_err();
        assert!(matches!(err, ValidationError::ZeroAmount));
    }

    #[test]
    fn test_validate_rejects_below_minimum() {
        let rules = ValidationRules {
            min_amount: TokenAmount::from_atto(1000u64),
            max_amount: TokenAmount::from_atto(0u64),
            allowed_tokens: vec![],
        };
        let event = make_event(make_addr(10), make_addr(20), 500);
        let err = rules.validate(&event).unwrap_err();
        assert!(matches!(err, ValidationError::AmountBelowMinimum { .. }));
    }

    #[test]
    fn test_validate_accepts_at_minimum() {
        let rules = ValidationRules {
            min_amount: TokenAmount::from_atto(1000u64),
            max_amount: TokenAmount::from_atto(0u64),
            allowed_tokens: vec![],
        };
        let event = make_event(make_addr(10), make_addr(20), 1000);
        assert!(rules.validate(&event).is_ok());
    }

    #[test]
    fn test_validate_rejects_above_maximum() {
        let rules = ValidationRules {
            min_amount: TokenAmount::from_atto(0u64),
            max_amount: TokenAmount::from_atto(500u64),
            allowed_tokens: vec![],
        };
        let event = make_event(make_addr(10), make_addr(20), 1000);
        let err = rules.validate(&event).unwrap_err();
        assert!(matches!(err, ValidationError::AmountAboveMaximum { .. }));
    }

    #[test]
    fn test_validate_accepts_at_maximum() {
        let rules = ValidationRules {
            min_amount: TokenAmount::from_atto(0u64),
            max_amount: TokenAmount::from_atto(500u64),
            allowed_tokens: vec![],
        };
        let event = make_event(make_addr(10), make_addr(20), 500);
        assert!(rules.validate(&event).is_ok());
    }

    #[test]
    fn test_validate_accepts_within_range() {
        let rules = ValidationRules {
            min_amount: TokenAmount::from_atto(100u64),
            max_amount: TokenAmount::from_atto(1000u64),
            allowed_tokens: vec![],
        };
        let event = make_event(make_addr(10), make_addr(20), 500);
        assert!(rules.validate(&event).is_ok());
    }

    #[test]
    fn test_validate_rejects_unlisted_token() {
        let allowed_token = make_addr(42);
        let rules = ValidationRules {
            min_amount: TokenAmount::from_atto(0u64),
            max_amount: TokenAmount::from_atto(0u64),
            allowed_tokens: vec![allowed_token],
        };
        let event = make_event(make_addr(99), make_addr(20), 100); // wrong token
        let err = rules.validate(&event).unwrap_err();
        assert!(matches!(err, ValidationError::TokenNotAllowed { .. }));
    }

    #[test]
    fn test_validate_accepts_listed_token() {
        let token = make_addr(42);
        let rules = ValidationRules {
            min_amount: TokenAmount::from_atto(0u64),
            max_amount: TokenAmount::from_atto(0u64),
            allowed_tokens: vec![token.clone()],
        };
        let event = make_event(token, make_addr(20), 100);
        assert!(rules.validate(&event).is_ok());
    }

    #[test]
    fn test_validate_empty_allowlist_accepts_any_token() {
        let rules = default_rules();
        let event = make_event(make_addr(999), make_addr(20), 100);
        assert!(rules.validate(&event).is_ok());
    }

    #[test]
    fn test_validate_rejects_zero_recipient() {
        let rules = default_rules();
        let event = make_event(make_addr(10), make_addr(0), 100); // id=0 is zero addr
        let err = rules.validate(&event).unwrap_err();
        assert!(matches!(err, ValidationError::InvalidRecipient));
    }

    // ── State: replay protection ──────────────────────────────────────────────

    #[test]
    fn test_replay_new_transfer_not_processed() {
        let (state, store) = make_state();
        let tid = make_transfer_id(1);
        assert!(!state.is_processed(&store, &tid).unwrap());
    }

    #[test]
    fn test_replay_marks_processed() {
        let (mut state, store) = make_state();
        let tid = make_transfer_id(2);
        assert!(!state.is_processed(&store, &tid).unwrap());
        state.mark_processed(&store, &tid, 100).unwrap();
        assert!(state.is_processed(&store, &tid).unwrap());
    }

    #[test]
    fn test_replay_different_ids_independent() {
        let (mut state, store) = make_state();
        let tid1 = make_transfer_id(1);
        let tid2 = make_transfer_id(2);
        state.mark_processed(&store, &tid1, 100).unwrap();
        assert!(state.is_processed(&store, &tid1).unwrap());
        assert!(!state.is_processed(&store, &tid2).unwrap());
    }

    #[test]
    fn test_replay_all_zeros_id() {
        let (mut state, store) = make_state();
        let tid = [0u8; 32];
        assert!(!state.is_processed(&store, &tid).unwrap());
        state.mark_processed(&store, &tid, 1).unwrap();
        assert!(state.is_processed(&store, &tid).unwrap());
    }

    #[test]
    fn test_replay_max_id() {
        let (mut state, store) = make_state();
        let tid = [0xFFu8; 32];
        state.mark_processed(&store, &tid, 999).unwrap();
        assert!(state.is_processed(&store, &tid).unwrap());
    }

    #[test]
    fn test_replay_multiple_marks() {
        let (mut state, store) = make_state();
        for i in 0u8..10 {
            let tid = make_transfer_id(i);
            state.mark_processed(&store, &tid, i as u64).unwrap();
        }
        for i in 0u8..10 {
            let tid = make_transfer_id(i);
            assert!(state.is_processed(&store, &tid).unwrap(), "tid {} should be processed", i);
        }
        // tid 10 not marked
        assert!(!state.is_processed(&store, &make_transfer_id(10)).unwrap());
    }

    // ── ValidationError display ───────────────────────────────────────────────

    #[test]
    fn test_validation_error_display_zero_amount() {
        let err = ValidationError::ZeroAmount;
        assert!(err.to_string().contains("zero"));
    }

    #[test]
    fn test_validation_error_display_duplicate() {
        let tid = make_transfer_id(5);
        let err = ValidationError::DuplicateTransfer { transfer_id: tid };
        assert!(err.to_string().contains("processed"));
    }

    // ── ValidationRules defaults ──────────────────────────────────────────────

    #[test]
    fn test_default_rules_accept_any_valid_event() {
        let rules = ValidationRules::default();
        // With 0 min/max and empty allowlist, any positive amount to a non-zero addr is ok
        let event = make_event(make_addr(1), make_addr(2), 1_000_000);
        assert!(rules.validate(&event).is_ok());
    }

    // ── State counters ────────────────────────────────────────────────────────

    #[test]
    fn test_state_initial_counters_zero() {
        let (state, _) = make_state();
        assert_eq!(state.relay_count, 0);
        assert_eq!(state.reject_count, 0);
    }
}
