//! MODEL-01: in-memory contract values, not adopted wire schemas.
//!
//! Opaque private fields intentionally prevent constructing fake validated state.
//! Replace each placeholder with the fields and ownership required by its contract;
//! do not use a byte blob, index alone or Default as a substitute for exact identity.
//! Spec: README.md#model-01

use bytes::Bytes;

/// Application storage key; its encoding must be decided with the backend.
pub type StateKey = Bytes;
/// Stored application value; absence is represented by Option, never an I/O error.
pub type StateValue = Bytes;

/// Pinned read-only capability for one exact state/runtime base.
///
/// MODEL-01: define the representation before implementing its producers/consumers.
/// Spec: README.md#model-01
pub struct StateView {
    _private: (),
}

/// Bounded ordered inputs, predecessor, runtime and accepted direct-job identity.
///
/// MODEL-01: define the representation before implementing its producers/consumers.
/// Spec: README.md#model-01
pub struct ExecutedRange {
    _private: (),
}

/// Deterministic application effects; contains no root, certificate or DB handle.
///
/// MODEL-01: define the representation before implementing its producers/consumers.
/// Spec: README.md#model-01
pub struct StateChanges {
    _private: (),
}

/// Prepared commitments and exact accepted signing-unit identity.
///
/// MODEL-01: define the representation before implementing its producers/consumers.
/// Spec: README.md#model-01
pub struct PreparedState {
    _private: (),
}

/// Usable pinned view plus exact prefix, runtime and state identity.
///
/// MODEL-01: define the representation before implementing its producers/consumers.
/// Spec: README.md#model-01
pub struct AppliedCheckpoint {
    _private: (),
}

/// Recoverable checkpoint, certified subject, original evidence and provenance.
///
/// MODEL-01: define the representation before implementing its producers/consumers.
/// Spec: README.md#model-01
pub struct DurableApplied {
    _private: (),
}

/// Epoch, exact ordered range, canonical base, runtime and full result commitments.
///
/// MODEL-01: define the representation before implementing its producers/consumers.
/// Spec: README.md#model-01
pub struct ExecutionStatement {
    _private: (),
}

/// Original untrusted statements, eligible signer identities and signatures.
///
/// MODEL-01: define the representation before implementing its producers/consumers.
/// Spec: README.md#model-01
pub struct ExecutionCertificate {
    _private: (),
}

/// One original signature over the full execution statement.
///
/// MODEL-01: define the representation before implementing its producers/consumers.
/// Spec: README.md#model-01
pub struct ExecutionAttestation {
    _private: (),
}

/// Claimed work bound to exact parent, runtime, ordered prefix and local attempt.
/// MODEL-01: construct only after the Execution owner rechecks and reserves the attempt.
/// Spec: README.md#model-01
pub struct Job {
    _private: (),
}

/// Untrusted original peer evidence; transport receipt does not authenticate its subject.
pub enum ResultEvidence {
    Attestation(ExecutionAttestation),
    Certificate(ExecutionCertificate),
}

/// Exact execution projection of App-retained canonical input, without native ACK tokens.
/// MODEL-01: bind full ordered contents, predecessor, runtime and bounded unit identity.
/// Spec: README.md#model-01
pub struct OrderedInput {
    _private: (),
}

/// Retained native canonical delivery; only Execution owns and releases these ACK tokens.
/// MODEL-01: bind exact ordering/base linkage and project execution input without token transfer.
/// Spec: README.md#model-01
pub(crate) struct CanonicalInput {
    pub ordered: OrderedInput,
    pub updates: Vec<commonware_consensus::multimmit::marshal::Update<crate::Block>>,
}

/// Untrusted state/output material bound to a candidate certified target.
/// MODEL-01: define ownership and exact identity before import/recovery implementation.
/// Spec: README.md#model-01
pub struct StateMaterial {
    _private: (),
}

/// Verified retained import material; not canonical writer authority.
/// MODEL-01: define ownership and exact identity before import/recovery implementation.
/// Spec: README.md#model-01
pub struct PreparedImport {
    _private: (),
}

/// Stored checkpoint candidate, original evidence and exact verification context.
/// MODEL-01: define ownership and exact identity before import/recovery implementation.
/// Spec: README.md#model-01
pub struct RecoveredState {
    _private: (),
}
