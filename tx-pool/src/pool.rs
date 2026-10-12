//! POOL-01: transaction admission, selection and durable cleanup.

use crate::{
    payload::{Payload, PayloadLimits},
    types::{LaneContext, PoolUpdate, SelectionContext},
};
use bytes::Bytes;
use thiserror::Error;

/// Owns bounded transaction retention and proposal selection.
pub struct TxPool {
    lanes: LaneContext,
}
/// Local lane classification; it neither selects a batch nor establishes inclusion.
pub enum TransactionClass {
    /// Valid and assigned to this producer lane; retain and share with peers.
    Selected,
    /// Valid but not assigned to this producer lane; retain and share with peers.
    Unselected,
    /// Invalid under agreed transaction validity rules; do not retain or relay the payload.
    Invalid,
}

/// Completed classification, with duplicate handling separate from lane assignment.
pub struct Admission {
    /// Computed locally; peer classification is never authoritative.
    pub classification: TransactionClass,
    /// The same valid transaction is already retained; avoid another insertion or relay loop.
    pub duplicate: bool,
}

/// Admission could not retain valid input; the caller keeps it for retry.
#[derive(Debug, Error)]
pub enum AdmissionError {
    /// Capacity pressure is not transaction invalidity or a lane classification.
    #[error("bounded transaction retention is full")]
    Backpressure,
}

impl TxPool {
    /// Creates a pool with the trusted local lane assignment and shared routing context.
    pub fn open(_lanes: LaneContext) -> Self {
        // POOL-01: Initialize bounded retention for both valid classes and peer sharing.
        // Spec: README.md#pool-lane-context-and-sharing
        todo!("POOL-01: construct lane-aware shared transaction retention");
    }
    /// Classifies input and retains both valid classes; duplicates return their current class.
    pub fn admit(&mut self, _transaction: Bytes) -> Result<Admission, AdmissionError> {
        // POOL-01: Wrong-lane input is Unselected, never Invalid; retain/share both valid classes.
        // Keep deduplication and capacity errors separate from the three transaction classes.
        // Spec: README.md#pool
        todo!("POOL-01: implement admission using the selected pool backend");
    }
    /// Returns a bounded payload while keeping selected transactions retained.
    pub fn select(&mut self, _context: &SelectionContext, _limits: &PayloadLimits) -> Payload {
        // POOL-01: Select only this lane's ready Selected entries; retain both valid classes.
        // Bind proposal context to the installed lane context and preserve stable payload bytes.
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
