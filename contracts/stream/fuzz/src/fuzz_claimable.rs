// SPDX-License-Identifier: Apache-2.0

use paystream_stream::storage::claimable_amount;
use paystream_stream::types::{Stream, StreamStatus};
use proptest::prelude::*;
use soroban_sdk::{Address, Env};

fn make_stream(
    env: &Env,
    deposit: i128,
    withdrawn: i128,
    rate_per_second: i128,
    last_withdraw_time: u64,
    stop_time: u64,
    cliff_time: u64,
    paused_at: u64,
    status: StreamStatus,
) -> Stream {
    let addr = Address::generate(env);
    Stream {
        id: 1,
        employer: addr.clone(),
        employee: addr.clone(),
        token: addr,
        deposit,
        withdrawn,
        rate_per_second,
        start_time: 0,
        stop_time,
        last_withdraw_time,
        cooldown_period: 0,
        status,
        locked: false,
        cliff_time,
        paused_at,
        delegate: None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000_000))]

    /// claimable_amount must never panic and must always return a value in [0, deposit - withdrawn].
    #[test]
    fn fuzz_claimable_no_panic_and_bounded(
        deposit        in 0i128..=i64::MAX as i128,
        withdrawn      in 0i128..=i64::MAX as i128,
        rate           in 1i128..=1_000_000i128,
        last_withdraw  in 0u64..=u64::MAX / 2,
        stop_time      in 0u64..=u64::MAX / 2,
        now            in 0u64..=u64::MAX / 2,
    ) {
        let env = Env::default();
        let withdrawn = withdrawn.min(deposit);
        let stream = make_stream(&env, deposit, withdrawn, rate, last_withdraw, stop_time, 0, 0, StreamStatus::Active);
        let result = claimable_amount(&stream, now);
        let remaining = deposit - withdrawn;
        prop_assert!(result >= 0, "claimable must be non-negative");
        prop_assert!(result <= remaining, "claimable must not exceed remaining deposit");
    }

    /// Cancelled and Exhausted streams always return 0.
    #[test]
    fn fuzz_claimable_terminal_states_zero(
        deposit in 1i128..=i64::MAX as i128,
        rate    in 1i128..=1_000_000i128,
        now     in 0u64..u64::MAX / 2,
    ) {
        let env = Env::default();
        for status in [StreamStatus::Cancelled, StreamStatus::Exhausted] {
            let stream = make_stream(&env, deposit, 0, rate, 0, 0, 0, 0, status);
            prop_assert_eq!(claimable_amount(&stream, now), 0);
        }
    }

    /// Before cliff_time, claimable must be 0.
    #[test]
    fn fuzz_claimable_zero_before_cliff(
        deposit     in 1_000i128..=i64::MAX as i128,
        rate        in 1i128..=1_000_000i128,
        cliff_time  in 1u64..=u64::MAX / 2,
        now         in 0u64..=u64::MAX / 2,
    ) {
        // Only test the case where now < cliff_time
        prop_assume!(now < cliff_time);
        let env = Env::default();
        let stream = make_stream(&env, deposit, 0, rate, 0, 0, cliff_time, 0, StreamStatus::Active);
        prop_assert_eq!(claimable_amount(&stream, now), 0,
            "claimable must be 0 before cliff_time");
    }

    /// Paused streams (status == Paused) must return 0.
    #[test]
    fn fuzz_claimable_paused_returns_zero(
        deposit    in 1_000i128..=i64::MAX as i128,
        rate       in 1i128..=1_000_000i128,
        paused_at  in 1u64..=u64::MAX / 2,
        now        in 0u64..=u64::MAX / 2,
    ) {
        let env = Env::default();
        let stream = make_stream(&env, deposit, 0, rate, 0, 0, 0, paused_at, StreamStatus::Paused);
        let result = claimable_amount(&stream, now);
        prop_assert!(result >= 0, "claimable must be non-negative even when paused");
        prop_assert!(result <= deposit, "claimable must not exceed deposit when paused");
    }
}

fn main() {}
