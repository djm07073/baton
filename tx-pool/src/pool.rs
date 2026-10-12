//! POOL-01: transaction admission, selection and durable cleanup.

use crate::{
    payload::{Payload, PayloadLimits},
    types::{PoolUpdate, SelectionContext},
};
use bytes::Bytes;

/// Owns bounded transaction retention and proposal selection.
pub struct TxPool {
    _private: (),
}
/// Admission classification; selection does not establish canonical inclusion.
pub enum Admission {
    Selected,
    Unselected,
    Duplicate,
    Rejected,
}

impl TxPool {
    /// Validates static rules and retains or rejects a transaction under pool policy.
    pub fn admit(&mut self, _transaction: Bytes) -> Admission {
        // POOL-01: Validate static rules, deduplicate and enforce retention bounds.
        // Spec: README.md#pool
        todo!("POOL-01: implement admission using the selected pool backend");
    }
    /// Returns a bounded payload while keeping selected transactions retained.
    pub fn select(&mut self, _context: &SelectionContext, _limits: &PayloadLimits) -> Payload {
        // POOL-01: Return stable bounded proposal bytes; selected transactions remain retained.
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
