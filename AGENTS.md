# Baton implementation instructions

## Authority and scope

- Read [developer assignments](docs/03-reference/07-development-order.md#developer-assignments), the task's `Spec:` links and [communication contracts](docs/03-reference/01-communication-contracts.md) before implementation. Current user decisions override historical notes.
- `network/src`, `tx-pool/src`, `alignment/src`, `execution/src` and `storage/src` are the implementation crates. Each crate README owns its component specification; `docs/01-overview`, `02-e2e` and `03-reference` own the shared architecture, end-to-end flows and references. `docs/research/` contains the paper, handoff and preserved historical evidence; it is not a competing implementation contract.
- This is an incomplete scaffold. Do not describe compilation, empty tests or `todo!()` code as a working protocol, E2E result or safety proof.
- Preserve the historical `docs/research/archive/` and `docs/research/assets/review/` contents. Update current documents and links instead of rewriting recorded evidence. Do not publish or push without user authorization.

## Implementing an assignment

1. Identify the task ID and prerequisites in the developer guide. Read the owning crate README, each linked contract and the pinned native source before editing. A source `Spec:` path resolves from its crate directory, not the workspace root.
2. Define the task's input/output identity, ownership, completion and failure conditions. MODEL-01 placeholders deliberately have no usable constructors; replace them with the specified data and capabilities, not opaque bytes or default success values.
3. Implement only that responsibility. Reuse native Engine/Marshal/codec/runtime/crypto/storage mechanisms. Keep private decomposition flexible; add no forwarding traits, block service, orderer or result service.
4. Leave undecided VM, codec, global comparator, certification-unit, key and backend policies explicit. An unresolved policy is a design dependency to resolve before implementing that path, not permission to invent a production default.
5. Replace each `todo!()` only with its complete contract. Do not silence it with `Ok(())`, `true`, `None`, an empty collection, dropped requests, or an automatic ACK. Keep the task ID and useful spec reference alongside completed code; remove only obsolete scaffolding comments.
6. Add focused deterministic tests for the task's acceptance cases, including malformed input, cancellation, bounded capacity and recovery where specified. Do not create tests that merely assert the stub panics.
7. Run the applicable checks below and record task IDs completed, tests run, remaining TODOs and open decisions. Remove temporary dead-code allowances as implemented modules become connected.

## Required invariants

- TxPool validates, deduplicates and retains valid transactions by type for sharing across lanes. Admission is independent of local proposal eligibility; select chooses eligible type pools from the proposal epoch/lane. Keep invalid-input errors separate from retryable backpressure, preserve transactions across assignment changes, and reconcile all types from durable outcomes across all lanes.
- `Automaton::verify(true)` means valid payload and durable custody. Missing data stays pending; speculative execution and result certification never delay the verdict.
- Retain every accepted Marshal Update and Exact token. `Reporter::report` performs a synchronous handoff; `Backoff` cannot recover a dropped Update.
- `Alignment::commit` records finalized ordering only. `next` selects; the Execution owner rechecks and claims exact parent/runtime/attempt/resources before launch. Preserve started F and sort only eligible pending work.
- The canonical worker awaits whole-unit execution/reuse/repair and `Storage::submit` acceptance before the next unit. Keep shared Execution locks and native callbacks out of those waits.
- Keep crate dependencies one-way: Network assembles Execution, Storage, TxPool and Alignment; Execution imports TxPool/Alignment and an injected Storage contract; the backend implements that Execution-owned contract. TxPool and Alignment use their own read-only inputs, never Execution internals. Execution retains native Update/ACK ownership. AlignmentPolicy::next is public for cross-crate selection; private policy helpers remain private.
- Execution alone verifies full-subject, distinct eligible `f+1` result evidence. `VerifiedCertificate` has no public constructor, decoder, Default or trusted deserialized flag. Imported work cannot become own direct-signing evidence.
- Storage owns roots, material checks, one canonical writer and durable records. Cold read errors are not absence. Acceptance is not durability; only matching `Committed` permits checkpoint publication and exactly covered ACKs. Failed mutable storage stops until recovery.
- App-only Baton cannot reorder Marshal Updates or implement authenticated native prefix adoption. That protocol extension is a separate open design task.

## Validation

Use `just check` (or its commands in README when `just` is unavailable), plus focused tests for implemented behavior. Protocol tests use Commonware's deterministic runtime and deterministic RNG. Use existing Commonware runtime traits and `commonware_macros::select!`; do not introduce a Tokio runtime into protocol modules.

Keep public interfaces in ALPHA stability scope. Use codec Buf for generic untrusted decoding, bounded codecs, cheap owned buffers and static dispatch. Use thiserror for errors. Avoid unsafe code. Keep one pinned Commonware dependency graph.

Each crate must inherit `[workspace.lints]` using `[lints] workspace = true`. Use the repository-root `rustfmt.toml` and `clippy.toml`; do not add per-crate copies or weaken shared rules. These files and the shared lint levels follow Commonware and are kept locally for standalone checkouts. Use nightly rustfmt via `just fix-fmt` / `just check-fmt`; the formatter must include every owned source file without rewriting research archives. Any temporary lint exception for a TODO belongs on that unfinished item, with its task ID and removal condition.

`python3 scripts/check_scaffold.py` checks TODO task IDs and specification anchors. `npm --prefix docs run docs:check` checks current documentation and rendered diagrams. Rebuild with `npm --prefix docs run docs:build`; edit source Markdown, not generated site files. These checks do not replace implementation tests or establish protocol correctness.
