//! Shared scheduling interface; no execution, storage I/O or result certification.

use crate::types::{Candidate, CandidateAdmission, JobIntent, OrderedView, SchedulingView};

/// Qualified candidate intake and finalized ordering, shared by both policies.
/// Spec: README.md#model-01
pub trait Alignment {
    /// Retains/deduplicates metadata without starting execution or fetching bodies.
    fn admit(&mut self, candidate: Candidate) -> CandidateAdmission;
    /// Records exact continuous ordering idempotently, without execution or durability waits.
    fn commit(&mut self, ordered: &OrderedView);
}

/// Selects from the owner's immutable view; the owner still must claim resources.
pub trait AlignmentPolicy: Alignment + Send + 'static {
    /// Selects an intent without claiming resources or starting execution.
    fn next(&self, view: &SchedulingView<'_>) -> Option<JobIntent>;
}

/// Scheduling mode selected during Node assembly.
#[derive(Clone, Copy)]
pub enum AlignmentMode {
    /// Speculation with native ordering and no Baton traffic.
    PreCut,
    /// Baton speculative policy; native protected-prefix integration is separate open work.
    Baton,
}

/// Pre-cut scheduling without Baton reports or advisory direction.
pub struct PreCutAlignment {
    _private: (),
}
/// Baton scheduling using the same candidate, ordering and execution-view contracts.
pub struct BatonAlignment {
    _private: (),
}

impl Alignment for PreCutAlignment {
    fn admit(&mut self, _candidate: Candidate) -> CandidateAdmission {
        // PRECUT-01: Deduplicate qualified inputs with bounded pending retention.
        // Spec: README.md#alignment-global-rule-for-local-execution-and-reports
        todo!("PRECUT-01: implement candidate admission");
    }
    fn commit(&mut self, _ordered: &OrderedView) {
        // PRECUT-01: Reflect immutable continuous order; no execution, f+1 or ACK wait.
        // Spec: README.md#precut-01
        todo!("PRECUT-01: implement ordering-only commit");
    }
}

impl AlignmentPolicy for PreCutAlignment {
    fn next(&self, _view: &SchedulingView<'_>) -> Option<JobIntent> {
        // PRECUT-01: Preserve F, sort eligible unstarted work only, and require an exact ready parent.
        // Spec: README.md#precut-01
        todo!("PRECUT-01: select an intent without claiming or launching it");
    }
}

impl Alignment for BatonAlignment {
    fn admit(&mut self, _candidate: Candidate) -> CandidateAdmission {
        // BATON-01: Use the same candidate contract; report/direction work stays internal.
        // Spec: README.md#alignment
        todo!("BATON-01: implement bounded Baton candidate admission");
    }
    fn commit(&mut self, _ordered: &OrderedView) {
        // BATON-01: Reflect native order immediately, independently of advisory reports.
        // Spec: README.md#baton-01
        todo!("BATON-01: implement ordering-only commit");
    }
}

impl AlignmentPolicy for BatonAlignment {
    fn next(&self, _view: &SchedulingView<'_>) -> Option<JobIntent> {
        // BATON-01: Select ready work using bounded authenticated report/direction policy.
        // Spec: README.md#alignment
        todo!("BATON-01: implement speculation without claiming native prefix adoption");
    }
}
