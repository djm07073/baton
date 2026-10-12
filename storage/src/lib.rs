//! Storage implementation for the Execution-owned persistence interface.
//! TODO operations remain unimplemented; acceptance is not certified durability.

use commonware_macros::{stability_mod, stability_scope};

stability_mod!(ALPHA, mod backend);

stability_scope!(ALPHA {
    pub use backend::Backend;
});
