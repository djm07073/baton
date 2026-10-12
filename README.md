# Baton

Rust implementation scaffold for an App connected to Commonware Multimmit and Marshal.
**The project compiles; `todo!()` operations are unimplemented and the node cannot run yet.**

Start with [developer assignments](docs/03-reference/07-development-order.md#developer-assignments), then the [architecture](docs/01-overview/01-architecture.md). Each implementation stub carries a task ID and a `Spec: README.md#...` comment resolved relative to its owning crate. [AGENTS.md](AGENTS.md) defines completion requirements.

| Crate | Responsibility and specification |
|---|---|
| [network](network/README.md) | Native network/Marshal assembly, configuration and runtime supervision |
| [tx-pool](tx-pool/README.md) | Transaction admission, retained selection, payload codec and durable maintenance |
| [alignment](alignment/README.md) | Pre-cut/Baton policies; qualified candidates, ordering and next-job selection |
| [execution](execution/README.md) | Native callbacks, coordination, execution, checkpoints, certification and ACK |
| [storage](storage/README.md) | Pinned reads, roots, certified persistence, import material and recovery |
| [docs](docs/README.md) | Reading order, architecture, E2E flows, shared references and research history |

The root is a virtual Cargo workspace. Network assembles the runtime; Execution imports TxPool and Alignment; Storage implements the Execution-owned storage contract. Execution is generic over that contract and never imports the concrete Storage backend. TxPool and Alignment define their own input projections without depending on Execution. There is one canonical README per crate, linked from the [contents](docs/SUMMARY.md).

## Validate a change

Use Rust 1.96+, nightly rustfmt, Node.js 20+, Python 3 and optionally `just`:

```sh
rustup component add rustfmt --toolchain nightly
find network/src tx-pool/src alignment/src execution/src storage/src -name '*.rs' -type f -print0 | xargs -0 rustfmt +nightly --edition 2024 --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 scripts/check_scaffold.py
npm --prefix docs ci
npm --prefix docs run docs:check
npm --prefix docs run docs:build
```

`just check` runs the Rust, scaffold and documentation checks; `just check-stability` checks the public ALPHA scope. `just docs` builds the documentation; `npm --prefix docs run docs:serve` serves it locally. All generated site files stay in `docs/site/`.

`rustfmt.toml`, `clippy.toml` and `[workspace.lints]` carry the Commonware monorepo rules locally so a standalone checkout uses the same settings. Cargo does not inherit lints across independent workspaces: every Baton crate must use `[lints] workspace = true`. Nested crates use the root formatter/Clippy files without adding their own copies. Update these root rules when adopting changes from Commonware; they are not live links to a parent checkout. Baton also forbids unsafe code and temporarily permits dead code in the unfinished scaffold.

Use `just fix-fmt`, `just check-fmt` and `just clippy` for focused checks. Formatting uses nightly because Commonware's import grouping options require it; compilation and Clippy use the normal Rust toolchain. `NIGHTLY_VERSION` and `RUSTFMT` override the formatting command as in Commonware. Pass every owned source file to rustfmt because `stability_mod!` declarations are not traversed by `cargo fmt`; keep archived sources under `docs/research/` out of formatting.

The Commonware git dependencies share the exact spec revision `6233438985d8249d2b2bc1204191d5d405652288`; commit `Cargo.lock` changes deliberately. The lock retains `blst` 0.3.16, matching this pin; 0.3.17 changes the Pairing API and does not compile with the pinned source. This workspace builds standalone or as the `baton` submodule. Sha256/MinPk types follow the native example wiring; transaction formats, result-signing keys/threshold scheme, pool/backend policy and native Baton extension remain undecided. No VM, wire protocol, persistence behavior or protected-prefix guarantee is implemented by this scaffold.
