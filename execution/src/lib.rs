//! Native application callbacks, shared coordination, execution and result verification.
//! TODO operations remain unimplemented; compilation does not establish a working node.

use commonware_macros::{stability_mod, stability_scope};

stability_mod!(ALPHA, pub mod storage);
stability_mod!(ALPHA, pub mod types);
stability_mod!(ALPHA, mod coordination);
stability_mod!(ALPHA, mod execution);

stability_scope!(ALPHA {
    pub use coordination::{Block, Execution, Marshal};
    pub use execution::VerifiedCertificate;
});
