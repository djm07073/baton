# Alignment interface and Mermaid flow review

Scope: local implementation documentation and diagram artifacts, following the user's interface-flow and Alignment naming requests. No native Rust implementation, runtime E2E result, protocol proof, benchmark or external publication is claimed.

The [interface map](../../../docs/overview/interfaces.md) distinguishes Consensus, Alignment layer, Executor and Storage inside App. Existing calls [C], App behavior [A], proposed traits [P] and missing native work [X] remain distinct. Public Alignment admits candidates and records ordering; internal AlignmentPolicy selects work for shared dispatch. PreCutAlignment and BatonAlignment implement both. Private Baton method decomposition stays unspecified.

Storage now explicitly defines read/submit/certify methods for pinned cold reads, exact ordered changes and certificate intake. Prepared/Committed/Failed events expose signing readiness, certified durability and scoped errors. Executor submits StateChanges with bound execution metadata and stays independent of physical batching. It installs committed checkpoints, skips covered work, preserves compatible descendants and safely retires memory. The retained owner ACKs Marshal only after certified durability. Signature collection stays in App result handling, and import/recovery uses the same storage writer and checkpoint handoff.

All 19 inline Mermaid sources and their MMD/SVG/PNG assets were regenerated. Visual review covered every diagram, with overlapping overview labels simplified. The renderer now serializes its SVG DOM as XML so line breaks in HTML labels remain valid standalone SVG images.

The [Mermaid-to-interface audit](mermaid-interface-audit.md) maps all 19 diagrams to their specification sections and records corrections to caller ownership, method results, Storage events, recovery and startup. Its [source hashes](mermaid-interface-audit.json) identify the reviewed versions. Every named call/event has a documented contract or an explicit unresolved marker; private helpers, logical wire messages and the four native [X] obligations are not settled public Rust interfaces.

Validation:

- `npm run docs:interfaces`: 5 existing Rust API excerpts exported; the proposed Alignment and Storage sketches are separate from the native reference.
- `npm run docs:check`: 32 content pages, 307 local links, 19 rendered diagrams, 38 blank policy cells; no failures.
- `npm run docs:build`: 32 pages, 19 diagrams and 3 source attachments.
- [Browser checks](browser/verification.json): 32 pages, 19 decoded images, 55 local targets, 32 fragment references and 3 byte-exact source downloads; search and mobile navigation checked. Seven screenshots are retained beside the report.
- `git diff --check`: clean.
- [Native source files](native-source-files.json): the 10 consulted files match native pin `6233438985d8249d2b2bc1204191d5d405652288`. Names and assembly order were checked against those sources; byte identity alone does not prove the proposed App integration.

The browser checker serves only the local generated site. Its paths reflect this workstation. Earlier review/publication records and historical snapshots were preserved.

## Completion audit

The final audit read all 19 current Mermaid sources and inspected their rendered diagrams. It checked the following requirements against the current worktree:

| Requirement | Evidence |
|---|---|
| Identify what developers implement and what is supplied | Interface map's developer-role table, existing callback catalog and proposed trait contracts |
| Separate Consensus, Alignment layer and App Executor | Architecture diagram and responsibility table; pool, workers and state remain inside App |
| Separate external calls from shared internal implementation requirements | Public `Alignment::admit/commit`, crate-internal `AlignmentPolicy::next`, both concrete implementations and visibility explanation |
| Leave Baton private function decomposition unspecified | No private report/plan/direction method catalog or method names in current docs; diagrams 06/07/12 describe behavior |
| Explain Multimmit entry points and execution timing | Diagrams 02/04/05/15 show propose, custody, verify and independent execution; diagrams 08/09 show ordering notification, reuse/repair, matching f+1 certification, durable application and ACK |
| Separate ordering notification from certified state application | `Alignment::commit(ordered)` runs without waiting for execution/certification; Storage gates canonical state application on matching f+1 and retains recoverable evidence; App uses signing units compatible with the delivery window |
| Cover remaining lifecycle and failure paths | Transaction intake 03, restart/startup 10/13, result certification 11, state import 14, native actors 16, QMDB/checkpoints 17/18 and independent PreCut 19 |
| Keep existing interfaces distinct from future protocol work | [C]/[A]/[P]/[X] notation, source-grounded native reference, and missing native adoption/validation/continuation obligations |
| Define the state-commit implementation contract | Public `Storage::read/submit/certify`, retained Prepared/Committed/Failed events, exact f+1/order/material verification, batching/replay/failure rules and unchanged Update-owner ACK responsibility |
| Specify cold reads, asynchronous handoff and checkpoint cleanup | Executor reads pinned cold state and submits exact ordered StateChanges; Storage prepares roots, batches certified persistence and emits a usable Committed checkpoint; diagrams 01/08/09/10/13/14/17/19 show installation and the ABC catch-up example |
| Deliver consistent, readable artifacts | All 19 inline sources match MMD and manifest hashes; SVG parses as XML; SVG/PNG site copies match; current diagrams visually inspected |
| Validate the current documentation, not stale output | Browser evidence source/HTML hashes match all 32 current pages; three downloaded source hashes match; ten native source files still match the stated pin; docs check and diff check pass |

This completes the documentation/interface-flow work. Native implementation and protocol proofs remain explicitly outside this documentation task; no native Rust or historical archive files were changed, and this batch has not been committed or published.
