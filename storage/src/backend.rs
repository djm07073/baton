//! State preparation, certified persistence, material import and recovery.

use baton_execution::{
    VerifiedCertificate,
    storage::{Storage, StorageError},
    types::*,
};

/// Implements the persistence interface; quorum verification remains Execution-owned.
pub struct Backend {
    _private: (),
}

impl Storage for Backend {
    async fn read(
        &self,
        _view: &StateView,
        _key: StateKey,
    ) -> Result<Option<StateValue>, StorageError> {
        // STORE-01: Pin/version the exact view through all readers; propagate unavailable/I/O errors.
        // Spec: README.md#store-01
        todo!("STORE-01: implement pinned cold reads with QMDB primitives");
    }
    async fn submit(
        &self,
        _executed: ExecutedRange,
        _changes: StateChanges,
    ) -> Result<(), StorageError> {
        // STORE-01: Validate authoritative ordering, retain idempotently and prepare deterministic roots.
        // Spec: README.md#store-01
        todo!("STORE-01: accept changes and emit Prepared independently of disk flush");
    }
    async fn commit(&self, _verified: VerifiedCertificate) -> Result<(), StorageError> {
        // STORE-01: Match verified subject/material/predecessor, persist through one writer, emit Committed.
        // Spec: README.md#persistence
        todo!("STORE-01: persist state, outputs, identity and original evidence recoverably");
    }
    /// Retains verified import material without granting canonical writer authority.
    async fn verify_material(
        &self,
        _verified: &VerifiedCertificate,
        _material: StateMaterial,
        _base: &AppliedCheckpoint,
    ) -> Result<Option<PreparedImport>, StorageError> {
        // IMPORT-01: Fetch bounded material using QMDB sync; verify full target/output applicability.
        // Retain isolated candidate material before commit; progress notifications are not durability.
        // Spec: README.md#import-01
        todo!("IMPORT-01: implement material retention and safe canonical-writer handoff");
    }
    /// Returns stored candidates and original evidence for Execution re-verification.
    async fn recover(&self) -> Result<RecoveredState, StorageError> {
        // IMPORT-01: Load candidates/evidence for Execution re-verification before publishing a checkpoint.
        // Spec: README.md#import-01
        todo!("IMPORT-01: recover exact state and evidence without recreating a trusted flag");
    }
}
