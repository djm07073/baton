//! POOL-01: transaction admission, selection and durable cleanup.

use crate::{
    payload::{Payload, PayloadLimits},
    types::{LaneContext, PoolUpdate, SelectionContext},
};
use bytes::Bytes;
use thiserror::Error;

/// Owns bounded transaction retention by type and proposal selection.
pub struct TxPool {
    lanes: LaneContext,
}

/// Successful validation and retention, independent of proposal eligibility.
pub struct Admission {
    /// The same valid transaction is already retained; avoid another insertion or relay loop.
    pub duplicate: bool,
}

/// Admission failed validation or could not retain valid input.
#[derive(Debug, Error)]
pub enum AdmissionError {
    /// Fails agreed transaction validity rules; do not retain or relay the payload.
    #[error("transaction is invalid")]
    Invalid,
    /// Valid input could not be retained; the caller keeps it for retry.
    #[error("bounded transaction retention is full")]
    Backpressure,
}

impl TxPool {
    /// Creates a pool with the trusted local lane assignment and shared routing context.
    pub fn open(_lanes: LaneContext) -> Self {
        // POOL-01: Initialize bounded retention by type and context-bound proposal selection.
        // Spec: README.md#pool-lane-context-and-sharing
        todo!("POOL-01: construct lane-aware shared transaction retention");
    }
    /// Validates and retains input by type, regardless of this lane's proposal eligibility.
    pub fn admit(&mut self, _transaction: Bytes) -> Result<Admission, AdmissionError> {
        // POOL-01: Validate, deduplicate and retain by type for sharing across lanes.
        // Defer epoch/lane eligibility to select; capacity pressure is retryable, not invalidity.
        // Spec: README.md#pool
        todo!("POOL-01: implement admission using the selected pool backend");
    }
    /// Selects eligible types for the proposal epoch/lane and returns retained bounded bytes.
    pub fn select(&mut self, _context: &SelectionContext, _limits: &PayloadLimits) -> Payload {
        // POOL-01: Resolve eligible type pools from the matching epoch/lane context, then pack ready txs.
        // Preserve retained transactions and stable payload bytes across cancellation or reassignment.
        // Spec: README.md#pool
        todo!("POOL-01: implement selection without canonical retirement");
    }
    /// Applies Execution-projected durable outcomes without adding an ACK gate.
    pub fn reconcile(&mut self, _applied: &PoolUpdate) {
        // POOL-01: Retire/refresh from durable outcomes without adding another ACK approval.
        // Spec: README.md#pool
        todo!("POOL-01: implement durable outcome maintenance");
    }
}
