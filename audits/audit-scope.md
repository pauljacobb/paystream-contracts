# PayStream Contracts — Audit Scope

**Issue:** [#285](https://github.com/Vera3289/paystream-contracts/issues/285)  
**Status:** Pending engagement  
**Commit baseline:** `1944ff587fcf98ca088f021fcf23e8d966697133`  
**Date:** 2026-05-31

---

## Objective

Engage a professional smart contract auditing firm to review the PayStream Soroban contracts prior to mainnet deployment. The audit must cover all contracts that handle user funds or enforce access control.

---

## Contracts in Scope

### 1. `contracts/stream` — Core salary streaming contract

| File | Lines | Description |
|---|---|---|
| `src/lib.rs` | 626 | Public entrypoints: create, withdraw, top-up, pause, resume, cancel |
| `src/storage.rs` | 381 | Persistence layer and claimable balance calculation |
| `src/access_control.rs` | 496 | Role-based and multisig authorization |
| `src/validate.rs` | 67 | Input validation helpers |
| `src/types.rs` | 183 | Domain models and storage keys |
| `src/events.rs` | 107 | On-chain event publishing |

### 2. `contracts/token` — Fungible payment token contract

| File | Lines | Description |
|---|---|---|
| `src/lib.rs` | 355 | SEP-41 token entrypoints |
| `src/storage.rs` | 54 | Token state persistence |
| `src/types.rs` | 12 | Token domain types |

**Total in-scope source lines (non-test):** ~2,281

---

## Out of Scope

- Test files (`src/test.rs`, `src/auth_tests.rs`, `src/multisig_tests.rs`, `src/prop_tests.rs`, `src/tests.rs`)
- Deployment scripts (`scripts/`)
- Frontend / SDK integration code

---

## Key Risk Areas

1. **Claimable balance calculation** — integer arithmetic in `storage.rs`; overflow or rounding errors could drain or lock funds.
2. **Access control** — employer/employee/admin authorization in `access_control.rs`; unauthorized state transitions must be impossible.
3. **Stream lifecycle transitions** — `Active → Paused → Active`, `Active → Cancelled`, `Active → Exhausted`; invalid transitions must revert.
4. **Token transfer atomicity** — `cancel_stream` splits deposit between employee and employer; partial-transfer scenarios must leave no funds stranded.
5. **Batch stream creation** — `create_streams_batch` must be fully atomic (all-or-nothing).
6. **Re-entrancy / cross-contract calls** — any external token calls must not allow re-entrant state manipulation.
7. **Stop-time and cliff-time edge cases** — boundary conditions around `stop_time`, `cliff_time`, and `cooldown_period`.

---

## Acceptance Criteria (from issue #285)

- [ ] Audit scope defined — **this document**
- [ ] All critical/high findings resolved before mainnet deployment
- [ ] Audit report published in `/audits` directory
- [ ] Re-audit scheduled after any significant contract changes

---

## Deliverables Expected from Auditor

1. Written report with findings classified by severity (Critical / High / Medium / Low / Informational)
2. Proof-of-concept for any critical or high findings
3. Remediation recommendations for each finding
4. Re-audit confirmation after fixes are applied

---

## Suggested Auditing Firms

- [OtterSec](https://osec.io/) — Soroban/Rust specialist
- [Trail of Bits](https://www.trailofbits.com/)
- [Halborn](https://halborn.com/)
- [Certik](https://www.certik.com/)
