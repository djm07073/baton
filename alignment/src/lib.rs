//! Candidate scheduling policy; no execution, storage I/O or native callback ownership.

use commonware_macros::{stability_mod, stability_scope};

stability_mod!(ALPHA, pub mod types);
stability_mod!(ALPHA, mod policy);

stability_scope!(ALPHA {
    pub use policy::{Alignment, AlignmentMode, AlignmentPolicy, BatonAlignment, PreCutAlignment};
});
