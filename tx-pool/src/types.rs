//! Pool inputs projected by Execution; no dependency on execution or storage internals.

/// Proposal context projected from the native callback, without changing its identity.
/// MODEL-01: carry producer/context identity and the agreed selection constraints.
/// Spec: README.md#model-01
pub struct SelectionContext {
    _private: (),
}

/// Durable transaction outcomes supplied by Execution for pool maintenance.
/// MODEL-01: bind exact applied unit, transaction identities and outcomes for replay-safe cleanup.
/// This value neither certifies state nor acknowledges Marshal delivery.
/// Spec: README.md#model-01
pub struct PoolUpdate {
    _private: (),
}
