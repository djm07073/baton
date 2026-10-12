//! Pool inputs supplied by Network and Execution, without importing their internals.

/// Trusted epoch, local producer lane, producer-lane set and shared assignment rule.
/// MODEL-01: a node without a producer lane retains valid transactions as Unselected.
/// Spec: README.md#pool-lane-context-and-sharing
pub struct LaneContext {
    _private: (),
}

/// Proposal context projected from the native callback, without changing its identity.
/// MODEL-01: bind proposal epoch/lane to the installed LaneContext and selection constraints.
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
