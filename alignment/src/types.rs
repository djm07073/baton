//! MODEL-01: in-memory contract values, not adopted wire schemas.
//!
//! Opaque private fields intentionally prevent constructing fake validated state.
//! Replace each placeholder with the fields and ownership required by its contract;
//! do not use a byte blob, index alone or Default as a substitute for exact identity.
//! Spec: README.md#model-01

/// Exact block/context plus validated custody and authentication facts.
///
/// MODEL-01: define the representation before implementing its producers/consumers.
/// Spec: README.md#model-01
pub struct Candidate {
    _private: (),
}

/// Speculative admission outcome; none of these authorizes execution or ACK.
pub enum CandidateAdmission {
    /// Metadata retained for later selection.
    Admitted,
    /// Exact candidate already known.
    AlreadyKnown,
    /// Speculation deferred under the selected resource policy.
    Deferred,
    /// Candidate currently ineligible for speculation.
    Ineligible,
}

/// Authoritative borrowed execution/resource projection, not another execution tree.
/// MODEL-01: include canonical/applied positions, F, checkpoints, claims and lifecycle.
/// Spec: README.md#model-01
pub struct SchedulingView<'a> {
    /// Exact authoritative finalized prefix.
    pub ordered: &'a OrderedView,
    /// Borrowed projection of Execution-owned state; policy cannot mutate execution.
    pub execution: &'a ExecutionView,
}

/// Selected input, exact ready parent and context; selecting does not claim a job.
/// MODEL-01: define exact identity and fencing before scheduler implementation.
/// Spec: README.md#model-01
pub struct JobIntent {
    _private: (),
}

/// Exact finalized order projected from retained Marshal input by Execution.
/// MODEL-01: bind context, continuous indices, exact block identity and preceding prefix.
/// Spec: README.md#model-01
pub struct OrderedView {
    _private: (),
}

/// Read-only applied base, started F, ready checkpoints, attempts and resource availability.
/// MODEL-01: project Execution state with exact identities, without a second mutable execution tree.
/// Spec: README.md#model-01
pub struct ExecutionView {
    _private: (),
}
