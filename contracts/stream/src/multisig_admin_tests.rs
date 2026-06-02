// SPDX-License-Identifier: Apache-2.0

//! Unit tests for M-of-N multi-sig admin (#275).

#![cfg(test)]

use soroban_sdk::{testutils::{Address as _, Ledger as _}, vec, Address, Env};
use crate::{StreamContract, StreamContractClient};
use crate::types::AdminOp;

fn setup() -> (Env, StreamContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &id);
    (env, client)
}

// ---------------------------------------------------------------------------
// configure_multisig
// ---------------------------------------------------------------------------

#[test]
fn test_configure_multisig() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let a3 = Address::generate(&env);

    client.initialize(&admin);
    client.configure_multisig(&admin, &vec![&env, a1.clone(), a2.clone(), a3.clone()], &2);

    let cfg = client.get_multisig_config();
    assert_eq!(cfg.threshold, 2);
    assert_eq!(cfg.admins.len(), 3);
}

#[test]
#[should_panic(expected = "threshold cannot exceed number of admins")]
fn test_configure_multisig_threshold_too_high() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let a1 = Address::generate(&env);

    client.initialize(&admin);
    client.configure_multisig(&admin, &vec![&env, a1.clone()], &2);
}

// ---------------------------------------------------------------------------
// multisig_propose
// ---------------------------------------------------------------------------

#[test]
fn test_multisig_propose_returns_id() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);

    client.initialize(&admin);
    client.configure_multisig(&admin, &vec![&env, a1.clone(), a2.clone()], &2);

    let op_id = client.multisig_propose(&a1, &AdminOp::EmergencyPause);
    assert_eq!(op_id, 1);
}

#[test]
#[should_panic(expected = "E022")]
fn test_multisig_propose_non_admin_rejected() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let a1 = Address::generate(&env);
    let outsider = Address::generate(&env);

    client.initialize(&admin);
    client.configure_multisig(&admin, &vec![&env, a1.clone()], &1);
    client.multisig_propose(&outsider, &AdminOp::EmergencyPause);
}

// ---------------------------------------------------------------------------
// multisig_approve — threshold logic
// ---------------------------------------------------------------------------

#[test]
fn test_threshold_not_met_does_not_execute() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let a3 = Address::generate(&env);

    client.initialize(&admin);
    client.configure_multisig(&admin, &vec![&env, a1.clone(), a2.clone(), a3.clone()], &3);

    let op_id = client.multisig_propose(&a1, &AdminOp::EmergencyPause);
    // Only 1 approval (proposer counts), threshold is 3 — not yet executed
    client.multisig_approve(&a2, &op_id);

    let pending = client.get_pending_admin_op(&op_id);
    assert!(!pending.executed);
    assert_eq!(pending.approvals.len(), 2);
}

#[test]
fn test_threshold_met_executes_emergency_pause() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);

    client.initialize(&admin);
    client.configure_multisig(&admin, &vec![&env, a1.clone(), a2.clone()], &2);

    let op_id = client.multisig_propose(&a1, &AdminOp::EmergencyPause);
    // a1 already approved via propose; a2 approval meets threshold
    client.multisig_approve(&a2, &op_id);

    let pending = client.get_pending_admin_op(&op_id);
    assert!(pending.executed);
}

#[test]
#[should_panic(expected = "E023")]
fn test_double_approve_rejected() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let a3 = Address::generate(&env);

    client.initialize(&admin);
    client.configure_multisig(&admin, &vec![&env, a1.clone(), a2.clone(), a3.clone()], &3);

    let op_id = client.multisig_propose(&a1, &AdminOp::EmergencyPause);
    client.multisig_approve(&a1, &op_id); // a1 already approved via propose
}

// ---------------------------------------------------------------------------
// Expiry
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "E024")]
fn test_expired_op_cannot_be_approved() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);

    client.initialize(&admin);
    client.configure_multisig(&admin, &vec![&env, a1.clone(), a2.clone()], &2);

    let op_id = client.multisig_propose(&a1, &AdminOp::EmergencyPause);

    // Advance time past the 7-day TTL
    env.ledger().with_mut(|l| l.timestamp += 7 * 24 * 3600 + 1);

    client.multisig_approve(&a2, &op_id);
}
