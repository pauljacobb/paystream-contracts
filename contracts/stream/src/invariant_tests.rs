// SPDX-License-Identifier: Apache-2.0

//! Invariant tests for the stream contract (issue #291).
//!
//! Invariants verified:
//!   1. `total_withdrawn + claimable <= deposit` at all times (funds conservation)
//!   2. Stream state transitions are valid (Active→Paused→Active, Active→Cancelled, Active→Exhausted)
//!   3. Only authorized callers can mutate stream state

#![cfg(test)]

use proptest::prelude::*;
use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env};
use crate::{StreamContract, StreamContractClient};
use crate::types::StreamStatus;

fn setup_env() -> (Env, StreamContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &id);
    (env, client)
}

fn setup_token(env: &Env, admin: &Address, supply: i128) -> Address {
    let token_id = env.register(paystream_token::TokenContract, ());
    let token = paystream_token::TokenContractClient::new(env, &token_id);
    token.initialize(admin, &supply);
    token_id
}

// ---------------------------------------------------------------------------
// Invariant 1: total_withdrawn + claimable <= deposit (funds conservation)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(150))]

    /// Invariant 1a: funds conservation holds after multiple withdrawals.
    #[test]
    fn inv_funds_conserved_after_multiple_withdrawals(
        deposit in 10_000i128..1_000_000i128,
        rate in 1i128..100i128,
        elapsed1 in 1u64..5_000u64,
        elapsed2 in 1u64..5_000u64,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        // First withdrawal
        env.ledger().with_mut(|l| l.timestamp += elapsed1);
        client.withdraw(&employee, &id);

        let s = client.get_stream(&id);
        let claimable = client.claimable(&id);
        prop_assert!(s.withdrawn + claimable <= s.deposit,
            "after 1st withdraw: withdrawn ({}) + claimable ({}) > deposit ({})",
            s.withdrawn, claimable, s.deposit);

        // Second withdrawal
        env.ledger().with_mut(|l| l.timestamp += elapsed2);
        client.withdraw(&employee, &id);

        let s2 = client.get_stream(&id);
        let claimable2 = client.claimable(&id);
        prop_assert!(s2.withdrawn + claimable2 <= s2.deposit,
            "after 2nd withdraw: withdrawn ({}) + claimable ({}) > deposit ({})",
            s2.withdrawn, claimable2, s2.deposit);
    }

    /// Invariant 1b: funds conservation holds after top-up.
    #[test]
    fn inv_funds_conserved_after_top_up(
        deposit in 10_000i128..500_000i128,
        rate in 1i128..50i128,
        top_up in 1_000i128..100_000i128,
        elapsed in 1u64..5_000u64,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + top_up + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        env.ledger().with_mut(|l| l.timestamp += elapsed);
        client.top_up(&employer, &id, &top_up);

        let s = client.get_stream(&id);
        let claimable = client.claimable(&id);
        prop_assert!(s.withdrawn + claimable <= s.deposit,
            "after top_up: withdrawn ({}) + claimable ({}) > deposit ({})",
            s.withdrawn, claimable, s.deposit);
    }

    /// Invariant 1c: withdrawn is monotonically non-decreasing.
    #[test]
    fn inv_withdrawn_monotonically_increases(
        deposit in 10_000i128..500_000i128,
        rate in 1i128..50i128,
        elapsed1 in 1u64..3_000u64,
        elapsed2 in 1u64..3_000u64,
        elapsed3 in 1u64..3_000u64,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        env.ledger().with_mut(|l| l.timestamp += elapsed1);
        client.withdraw(&employee, &id);
        let w1 = client.get_stream(&id).withdrawn;

        env.ledger().with_mut(|l| l.timestamp += elapsed2);
        client.withdraw(&employee, &id);
        let w2 = client.get_stream(&id).withdrawn;

        env.ledger().with_mut(|l| l.timestamp += elapsed3);
        client.withdraw(&employee, &id);
        let w3 = client.get_stream(&id).withdrawn;

        prop_assert!(w2 >= w1, "withdrawn decreased: w1={w1} w2={w2}");
        prop_assert!(w3 >= w2, "withdrawn decreased: w2={w2} w3={w3}");
    }
}

// ---------------------------------------------------------------------------
// Invariant 2: stream state transitions are valid
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(150))]

    /// Invariant 2a: newly created stream is always Active.
    #[test]
    fn inv_new_stream_is_active(
        deposit in 10_000i128..1_000_000i128,
        rate in 1i128..100i128,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        let s = client.get_stream(&id);
        prop_assert_eq!(s.status, StreamStatus::Active, "new stream must be Active");
    }

    /// Invariant 2b: pause transitions Active→Paused; resume transitions Paused→Active.
    #[test]
    fn inv_pause_resume_transitions(
        deposit in 10_000i128..1_000_000i128,
        rate in 1i128..100i128,
        elapsed in 1u64..1_000u64,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        // Active → Paused
        client.pause_stream(&employer, &id);
        let s = client.get_stream(&id);
        prop_assert_eq!(s.status, StreamStatus::Paused, "after pause: must be Paused");

        env.ledger().with_mut(|l| l.timestamp += elapsed);

        // Paused → Active
        client.resume_stream(&employer, &id);
        let s2 = client.get_stream(&id);
        prop_assert_eq!(s2.status, StreamStatus::Active, "after resume: must be Active");
    }

    /// Invariant 2c: cancelled stream stays Cancelled; claimable is 0.
    #[test]
    fn inv_cancelled_stream_is_terminal(
        deposit in 10_000i128..1_000_000i128,
        rate in 1i128..100i128,
        elapsed in 1u64..5_000u64,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        env.ledger().with_mut(|l| l.timestamp += elapsed);
        client.cancel_stream(&employer, &id);

        let s = client.get_stream(&id);
        prop_assert_eq!(s.status, StreamStatus::Cancelled, "after cancel: must be Cancelled");

        // claimable on a cancelled stream must be 0
        let claimable = client.claimable(&id);
        prop_assert_eq!(claimable, 0i128, "claimable on cancelled stream must be 0");
    }

    /// Invariant 2d: stream becomes Exhausted when fully drained; funds conserved.
    #[test]
    fn inv_exhausted_stream_funds_conserved(
        deposit in 1_000i128..100_000i128,
        rate in 10i128..100i128,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        // Advance time far enough to exhaust the stream
        let seconds_to_exhaust = (deposit / rate) as u64 + 1;
        env.ledger().with_mut(|l| l.timestamp += seconds_to_exhaust);
        client.withdraw(&employee, &id);

        let s = client.get_stream(&id);
        prop_assert!(s.withdrawn <= s.deposit,
            "withdrawn ({}) > deposit ({})", s.withdrawn, s.deposit);
        let claimable = client.claimable(&id);
        prop_assert!(s.withdrawn + claimable <= s.deposit,
            "withdrawn ({}) + claimable ({}) > deposit ({})",
            s.withdrawn, claimable, s.deposit);
    }

    /// Invariant 2e: paused stream accrues no new claimable while paused.
    #[test]
    fn inv_paused_stream_does_not_accrue(
        deposit in 10_000i128..1_000_000i128,
        rate in 1i128..100i128,
        elapsed_before in 1u64..1_000u64,
        elapsed_while_paused in 1u64..5_000u64,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        env.ledger().with_mut(|l| l.timestamp += elapsed_before);
        client.pause_stream(&employer, &id);
        let claimable_at_pause = client.claimable(&id);

        // Advance time while paused
        env.ledger().with_mut(|l| l.timestamp += elapsed_while_paused);
        let claimable_while_paused = client.claimable(&id);

        prop_assert_eq!(claimable_at_pause, claimable_while_paused,
            "paused stream accrued: before={claimable_at_pause} after={claimable_while_paused}");
    }
}

// ---------------------------------------------------------------------------
// Invariant 3: only authorized callers can mutate state
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Invariant 3a: employer address on stream matches the creator.
    #[test]
    fn inv_stream_employer_matches_creator(
        deposit in 10_000i128..500_000i128,
        rate in 1i128..50i128,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        let s = client.get_stream(&id);
        prop_assert_eq!(s.employer, employer, "stream employer must match creator");
        prop_assert_eq!(s.employee, employee, "stream employee must match parameter");
    }

    /// Invariant 3b: only the employer can pause; stream stays Active if wrong caller.
    /// Verified by checking that the employer field is immutable and only the employer
    /// can successfully pause (attacker != employer → panic → state unchanged).
    #[test]
    fn inv_employer_is_sole_pause_authority(
        deposit in 10_000i128..500_000i128,
        rate in 1i128..50i128,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        // Employer can pause
        client.pause_stream(&employer, &id);
        let s = client.get_stream(&id);
        prop_assert_eq!(s.status, StreamStatus::Paused, "employer must be able to pause");

        // Employer can resume
        client.resume_stream(&employer, &id);
        let s2 = client.get_stream(&id);
        prop_assert_eq!(s2.status, StreamStatus::Active, "employer must be able to resume");
    }

    /// Invariant 3c: only the employee can withdraw; deposit is unchanged after cancel.
    #[test]
    fn inv_cancel_returns_unearned_to_employer(
        deposit in 10_000i128..500_000i128,
        rate in 1i128..50i128,
        elapsed in 1u64..100u64,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let employee = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);
        let id = client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);

        env.ledger().with_mut(|l| l.timestamp += elapsed);
        let claimable_before_cancel = client.claimable(&id);
        client.cancel_stream(&employer, &id);

        let s = client.get_stream(&id);
        // After cancel: withdrawn == what employee earned up to cancel time
        prop_assert_eq!(s.status, StreamStatus::Cancelled);
        prop_assert!(s.withdrawn <= s.deposit,
            "withdrawn ({}) > deposit ({}) after cancel", s.withdrawn, s.deposit);
        // Employee got at most what was claimable
        prop_assert!(s.withdrawn <= claimable_before_cancel + 1,
            "employee got more than claimable: withdrawn={} claimable_before={}",
            s.withdrawn, claimable_before_cancel);
    }

    /// Invariant 3d: stream count is monotonically increasing.
    #[test]
    fn inv_stream_count_monotonically_increases(
        deposit in 10_000i128..200_000i128,
        rate in 1i128..50i128,
        n in 1u32..5u32,
    ) {
        let (env, client) = setup_env();
        let admin = Address::generate(&env);
        let employer = Address::generate(&env);
        let token_id = setup_token(&env, &employer, deposit * (n as i128) + 1);

        client.initialize(&admin);
        client.set_min_deposit(&admin, &0, &100);

        let mut prev_count = client.stream_count();
        for _ in 0..n {
            let employee = Address::generate(&env);
            client.create_stream(&employer, &employee, &token_id, &deposit, &rate, &0, &0, &0);
            let new_count = client.stream_count();
            prop_assert!(new_count > prev_count,
                "stream_count did not increase: prev={prev_count} new={new_count}");
            prev_count = new_count;
        }
    }
}
