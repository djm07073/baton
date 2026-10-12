//! Transaction execution, in-memory checkpoints and f+1 result verification.

use crate::{storage::Storage, types::*};
use baton_alignment::types::OrderedView;
use thiserror::Error;

/// Authority produced only after full local verification; never decode or Default this type.
/// Persist the original evidence, then reverify on recovery.
/// Spec: README.md#model-01
pub struct VerifiedCertificate {
    statement: ExecutionStatement,
    original: ExecutionCertificate,
}

impl VerifiedCertificate {
    /// Returns the exact locally verified subject for Storage material/predecessor matching.
    pub const fn statement(&self) -> &ExecutionStatement {
        &self.statement
    }
    /// Returns original evidence for durable retention and subsequent re-verification.
    pub const fn evidence(&self) -> &ExecutionCertificate {
        &self.original
    }
}

/// Execution/certification failures cannot authorize persistence or ACK.
#[derive(Debug, Error)]
pub(crate) enum ExecutionError {
    #[error("execution attempt failed or is stale")]
    FailedAttempt,
    #[error("execution evidence is invalid for this exact canonical context")]
    InvalidEvidence,
}

/// Direct computation evidence retained by Execution; imports cannot construct it.
/// MODEL-01: bind the accepted job and completed output, with stale-attempt fencing.
/// Spec: README.md#model-01
pub(crate) struct DirectEvidence {
    _private: (),
}

/// Owns exact-parent execution, memory checkpoints and result verification.
pub(crate) struct Worker {
    _private: (),
}

impl Worker {
    #[expect(
        clippy::unused_async,
        reason = "EXEC-01: remove when execution awaits cold reads"
    )]
    /// Executes claimed work using overlays and exact pinned cold reads.
    pub(crate) async fn execute(
        &self,
        _job: Job,
        _storage: &impl Storage,
    ) -> Result<(StateChanges, DirectEvidence), ExecutionError> {
        // EXEC-01: Use exact-parent overlays then pinned cold reads; fence duplicate/stale completion.
        // Spec: README.md#execution-exact-parents-and-reusable-work
        todo!("EXEC-01: implement deterministic execution and memory checkpoints");
    }
    #[expect(
        clippy::unused_async,
        reason = "EXEC-01: remove when reconciliation awaits execution"
    )]
    /// Completes reuse or repair for the full ordered unit; App retains native ACK tokens.
    pub(crate) async fn reconcile(
        &mut self,
        _input: &OrderedInput,
        _storage: &impl Storage,
    ) -> Result<(ExecutedRange, StateChanges), ExecutionError> {
        // EXEC-01: Reuse/finish/repair the complete ordered unit; never include unordered speculative work.
        // Spec: README.md#execution-execution-to-storage-completion
        todo!("EXEC-01: produce exact ordered changes with retained direct evidence");
    }
    /// Installs a usable durable base and skips only exactly covered work.
    pub(crate) fn install_checkpoint(&mut self, _checkpoint: &AppliedCheckpoint) {
        // EXEC-01: Skip covered work, retire conflicts, retain compatible descendants and pinned readers.
        // Spec: README.md#exec-01
        todo!("EXEC-01: install usable durable progress without another ACK round");
    }
    /// Signs matching prepared results only when supported by own direct execution.
    pub(crate) fn sign(
        &self,
        _prepared: &PreparedState,
        _direct: &DirectEvidence,
    ) -> Result<ExecutionAttestation, ExecutionError> {
        // CERT-01: Sign exact irrevocable order/base/runtime/full result only for own completed work.
        // Spec: README.md#certification-result-certification-inside-execution
        todo!("CERT-01: bind direct evidence and prepared commitments before signing");
    }
    /// Verifies full-subject evidence and returns authority only after eligible f+1 support.
    pub(crate) fn accept(
        &mut self,
        _evidence: ResultEvidence,
        _ordered: &OrderedView,
    ) -> Result<Option<VerifiedCertificate>, ExecutionError> {
        // CERT-01: Validate full subject, original signatures and distinct eligible f+1 identities.
        // Pending evidence returns None; only verified evidence can construct VerifiedCertificate.
        // Spec: README.md#certification-result-certification-inside-execution
        todo!("CERT-01: collect and verify evidence for direct work, import and recovery");
    }
}
