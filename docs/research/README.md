# Baton research and history

Start with **[Architecture](../01-overview/01-architecture.md)**. The implementation docs are organized in reading order: `docs/01-overview/` → crate READMEs → `docs/02-e2e/`; use `docs/03-reference/` for details. The [reading guide](../README.md) and [full contents](../SUMMARY.md) follow the same order. The integration starts with one App connected to the existing Multimmit Engine and Marshal.

| Start here | Covers |
|---|---|
| [Implementation docs](../README.md) · [Contents](../SUMMARY.md) | App-owned pool/schedulers/execution, existing native connections, E2E flows and open decisions |
| [Rust interfaces](../03-reference/02-rust-signatures.md) · [Single Rust file](../assets/interfaces/baton.rs) | Existing Commonware API excerpts for Automaton, Relay, Reporter and Marshal |
| [Pre-cut baseline implementation](../03-reference/06-precut-baseline.md) · [GitBook](https://app.gitbook.com/s/pvyFEde12m2tVRjI8TRw/baselines/precut) | Separate App scheduler mode without Baton: speculation, native ordering and incremental repair |
| [Research paper draft](baton-paper.md) | Algorithm, conditional safety/reuse arguments, native integration obligations, evaluation plan |

The [GitBook Baton space](https://app.gitbook.com/o/Z5g7kwPjokG0jEOyXNu6/s/pvyFEde12m2tVRjI8TRw/) is private within Beaker. [Rust interfaces](https://app.gitbook.com/s/pvyFEde12m2tVRjI8TRw/overview/rust-interfaces) is a direct entry point. This App-owned integration rewrite has not been published there; read docs/ for the current design. Git Sync and automatic Google Docs synchronization are not configured.

This is design work. Protocol implementation, native integration proofs, E2E execution, and benchmarks are not complete. Existing Commonware APIs and proposed application contracts stay distinct. Open policy cells remain blank. The Bank application model is outside the current scope.

Older documents and research material are preserved through the [archive index](archive/2026-10-03-root-history/README.md). Earlier implementation contracts are [archived](archive/2026-10-03-root-history/repository/research/archive/2026-10-03-implementation-outline/README.md). Historical Overpass documents are provenance, not replacements for the current design. [BATON_HANDOFF.md](BATON_HANDOFF.md) records sources and prior Google Doc revisions; [AGENTS.md](AGENTS.md) gives continuation instructions.

Current integration source: [Multimmit and Marshal at 6233438985d8249d2b2bc1204191d5d405652288](https://github.com/0xEyrie/monorepo/tree/6233438985d8249d2b2bc1204191d5d405652288). The paper and historical reviews retain their earlier Commonware source pin. [Tempo Technical Overview](https://app.notion.com/p/2dfc1352439b801db5b6cf6fe21fc315?pvs=204) and [Tempo DeepWiki](https://deepwiki.com/tempoxyz/tempo) are references for presentation structure. Their Simplex / Reth / REVM implementation is not adopted.

The canonical repository is [djm07073/baton](https://github.com/djm07073/baton). Application source lives in `src/`; current specifications and this research collection live under `docs/`.

## Local preview

Use Node.js 20+ and Python 3 from the repository root:

```sh
npm --prefix docs ci
npm --prefix docs run docs:check
npm --prefix docs run docs:build
npm --prefix docs run docs:serve
```

The [local preview](http://127.0.0.1:8765) provides navigation, search, an in-page outline, and previous/next links. docs/site/ is generated and excluded from Git. Edit docs/. Mermaid source and rendered assets live in docs/assets/diagrams/; diagram changes require refreshed renders and manifest.

[IMPLEMENTATION_SPEC.md](IMPLEMENTATION_SPEC.md) preserves older section links. The [pre-partition snapshot](archive/2026-10-03-before-gitbook/README.md) retains original prose and diagrams. Archived snapshots are historical and stay unchanged. [GitBook publication verification](assets/review/gitbook-upload-20261003/README.md) records the initial upload; the [English revision checks](assets/review/english-docs-20261003/README.md) record the translated update.
