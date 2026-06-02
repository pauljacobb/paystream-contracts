// SPDX-License-Identifier: Apache-2.0

use soroban_sdk::{contracttype, Address};

/// Status of a salary stream.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum StreamStatus {
    Active,
    Paused,
    Cancelled,
    Exhausted,
}

/// A salary stream: employer deposits funds, employee withdraws per-second.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Stream {
    pub id: u64,
    pub employer: Address,
    pub employee: Address,
    pub token: Address,
    pub deposit: i128,
    pub withdrawn: i128,
    pub rate_per_second: i128,
    pub start_time: u64,
    pub stop_time: u64,
    pub last_withdraw_time: u64,
    pub cooldown_period: u64,
    pub status: StreamStatus,
    pub locked: bool,
    /// Optional cliff: no tokens claimable before this timestamp (0 = no cliff). (#123)
    pub cliff_time: u64,
    /// Timestamp when the stream was last paused (0 if not paused). (#123 / pause fix)
    pub paused_at: u64,
    /// Optional delegate address authorized to manage the stream. (#287)
    pub delegate: Option<Address>,
}

/// Record of a pause/resume event for history tracking.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PauseEvent {
    pub stream_id: u64,
    pub timestamp: u64,
    pub is_pause: bool, // true for pause, false for resume
}

/// Parameters for a single stream in a batch create call.
#[contracttype]
#[derive(Clone, Debug)]
pub struct StreamParams {
    pub employee: Address,
    pub token: Address,
    pub deposit: i128,
    pub rate_per_second: i128,
    pub stop_time: u64,
    /// Optional cliff timestamp (0 = no cliff). (#123)
    pub cliff_time: u64,
}

// ---------------------------------------------------------------------------
// Governance types (#124)
// ---------------------------------------------------------------------------

/// Which protocol parameter a governance proposal targets.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum GovParam {
    MinDeposit,
    MaxDuration,
    FeeBps,
}

/// State of a governance proposal.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Executed,
    Rejected,
}

/// An on-chain governance proposal.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    pub id: u64,
    pub param: GovParam,
    pub new_value: u64,
    pub votes_for: u64,
    pub votes_against: u64,
    pub status: ProposalStatus,
    /// Ledger timestamp after which the proposal can be executed (timelock).
    pub executable_after: u64,
}

// ---------------------------------------------------------------------------
// Packed config (#272) — all small scalar config fields in one ledger entry
// to reduce the number of instance-storage reads per hot-path call.
//
// Layout:
//   min_deposit         i128  — minimum deposit for stream creation
//   fee_bps             u32   — protocol fee in basis points (0–100)
//   max_streams         u32   — max streams per employer
//   admin_nonce         u64   — replay-protection nonce for admin ops
//   paused              bool  — global contract pause flag
// ---------------------------------------------------------------------------
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContractConfig {
    pub min_deposit: i128,
    pub fee_bps: u32,
    pub max_streams: u32,
    pub admin_nonce: u64,
    pub paused: bool,
}

impl ContractConfig {
    pub fn default() -> Self {
        ContractConfig {
            min_deposit: 10_000,
            fee_bps: 0,
            max_streams: 100,
            admin_nonce: 0,
            paused: false,
        }
    }
}

/// Storage keys.
#[contracttype]
pub enum DataKey {
    Stream(u64),
    StreamCount,
    Admin,
    MinDeposit,
    AdminNonce,
    Paused,
    EmployerStreams(Address),
    EmployeeStreams(Address),
    PendingAdmin,
    FeeBps,
    FeeRecipient,
    /// Pending employer for a two-step stream ownership transfer.
    PendingEmployer(u64),
    /// Maximum number of streams an employer can create.
    MaxStreamsPerEmployer,
    /// Pause history for a stream.
    PauseHistory(u64),
    // Governance (#124)
    Proposal(u64),
    ProposalCount,
    Voted(u64, Address),
    // Token allowlist (#292)
    AllowedToken(Address),
    AllowedTokens,
    // Packed config (#272) — replaces individual MinDeposit/FeeBps/MaxStreamsPerEmployer/AdminNonce/Paused keys
    Config,
    // Multi-sig admin (#275)
    MultisigConfig,
    PendingAdminOp(u64),
    PendingAdminOpCount,
}

pub const ERR_ZERO_RATE: &str = "E001: rate_per_second must be greater than zero";
pub const ERR_ZERO_DEPOSIT: &str = "E002: deposit must be positive";
pub const ERR_REENTRANT: &str = "E003: reentrant withdraw detected";
pub const ERR_OVERFLOW: &str = "E004: arithmetic overflow in claimable calculation";
pub const ERR_STREAM_CANCELLED: &str = "E005: cannot top up a cancelled stream";
pub const ERR_STREAM_EXHAUSTED: &str = "E006: cannot top up an exhausted stream";
pub const ERR_BELOW_MIN_DEPOSIT: &str = "E007: deposit below minimum";
pub const ERR_INVALID_RATE: &str = "E008: rate_per_second exceeds maximum";
pub const ERR_BAD_NONCE: &str = "E009: invalid admin nonce";
pub const ERR_SAME_PARTY: &str = "E010: employer and employee must differ";
pub const ERR_FEE_TOO_HIGH: &str = "E011: fee_bps exceeds maximum of 100";
pub const ERR_INVALID_TOKEN: &str = "E012: token address is not a valid SEP-41 contract";
pub const ERR_UNAUTHORIZED_TRANSFER: &str = "E013: not the pending employer for this stream";
pub const ERR_DURATION_TOO_LONG: &str = "E014: stream duration exceeds maximum allowed";
pub const ERR_STOP_TIME_PAST: &str = "E016: stop_time must be in the future";
pub const ERR_MAX_STREAMS_REACHED: &str = "E015: maximum streams per employer reached";
pub const ERR_WITHDRAW_COOLDOWN: &str = "E010: withdraw cooldown not expired";
pub const ERR_ALREADY_PAUSED: &str = "E016: stream is already paused";
pub const ERR_NOT_PAUSED: &str = "E017: stream is not paused";
pub const ERR_TOKEN_NOT_ALLOWED: &str = "E018: token is not on the allowlist";
pub const ERR_CLIFF_AFTER_STOP: &str = "E019: cliff time must be before or equal to stop time";

// ---------------------------------------------------------------------------
// Multi-sig admin types (#275)
// ---------------------------------------------------------------------------

/// Configuration for M-of-N admin multi-sig.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MultisigConfig {
    /// Ordered list of admin addresses.
    pub admins: soroban_sdk::Vec<Address>,
    /// Number of signatures required to approve an operation.
    pub threshold: u32,
}

/// A sensitive operation pending multi-sig approval.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum AdminOp {
    /// Upgrade the contract WASM to the given hash.
    Upgrade(soroban_sdk::BytesN<32>),
    /// Trigger an emergency global pause.
    EmergencyPause,
}

/// A pending admin operation awaiting threshold signatures.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PendingAdminOp {
    pub id: u64,
    pub op: AdminOp,
    /// Addresses that have already approved.
    pub approvals: soroban_sdk::Vec<Address>,
    /// Ledger timestamp after which this pending op expires.
    pub expires_at: u64,
    pub executed: bool,
}

pub const ERR_MULTISIG_NOT_CONFIGURED: &str = "E021: multisig admin not configured";
pub const ERR_NOT_MULTISIG_ADMIN: &str = "E022: caller is not a multisig admin";
pub const ERR_ALREADY_APPROVED: &str = "E023: caller already approved this operation";
pub const ERR_OP_EXPIRED: &str = "E024: pending operation has expired";
pub const ERR_OP_ALREADY_EXECUTED: &str = "E025: pending operation already executed";
pub const ERR_THRESHOLD_NOT_MET: &str = "E026: approval threshold not yet met";
pub const ERR_OP_NOT_FOUND: &str = "E027: pending operation not found";
