//! Pinned reads, state material and certified persistence; quorum verification is Execution-owned.

use crate::{VerifiedCertificate, types::*};
use std::future::Future;
use thiserror::Error;

/// App storage contract. Acceptance is separate from certified durability.
/// Spec: README.md#model-01
pub trait Storage: Send + Sync {
    /// Reads the exact pinned base. None is verified absence; errors never select a newer base.
    fn read(
        &self,
        view: &StateView,
        key: StateKey,
    ) -> impl Future<Output = Result<Option<StateValue>, StorageError>> + Send;
    /// Retains a complete ordered unit; Ok allows the canonical worker to take the next unit.
    fn submit(
        &self,
        executed: ExecutedRange,
        changes: StateChanges,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;
    /// Accepts verified authority for matching material; only Committed authorizes durable progress.
    fn commit(
        &self,
        verified: VerifiedCertificate,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;
    /// Retains applicable import material without advancing canonical state.
    fn verify_material(
        &self,
        verified: &VerifiedCertificate,
        material: StateMaterial,
        base: &AppliedCheckpoint,
    ) -> impl Future<Output = Result<Option<PreparedImport>, StorageError>> + Send;
    /// Loads state and original evidence for Execution re-verification before publication.
    fn recover(&self) -> impl Future<Output = Result<RecoveredState, StorageError>> + Send;
}

/// Retained events delivered through the Execution owner using existing channels.
pub enum StorageEvent {
    /// Exact-unit commitments ready for direct signing.
    Prepared(PreparedState),
    /// Certified durable checkpoint ready for publication and covered ACKs.
    Committed(DurableApplied),
    /// Scoped failure; no durable completion is implied.
    Failed(StorageFailure),
}

/// Synchronous failures keep input pending or require storage-instance recovery.
#[derive(Debug, Error)]
pub enum StorageError {
    /// The caller remains responsible for retryable work before acceptance.
    #[error("bounded retention cannot accept this submission")]
    Backpressure,
    /// Cannot read this exact base; do not report absence.
    #[error("pinned view is unavailable")]
    UnavailableView,
    /// Exact order, material or predecessor does not match.
    #[error("request does not apply to retained material and current predecessor")]
    Inapplicable,
    /// The mutable instance must stop and recover.
    #[error("storage instance failed; recovery is required")]
    Fatal,
}

/// MODEL-01: include exact unit/instance scope and error before emitting failures.
/// Spec: README.md#model-01
pub struct StorageFailure {
    _private: (),
}
