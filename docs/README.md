# Baton Docs

**Understand the components, their responsibilities and what they exchange; then follow the same connections through the E2E flows.** One App connects to the existing Multimmit Engine and Marshal. Inside App, the pool, Alignment, Executor, result processing and Storage each own a part of that flow.

## Read in this order

1. **Overview:** read [Architecture](overview/architecture.md) for the whole composition and [Roles and terminology](overview/glossary.md) for the identities used between components.
2. **Components and communication:** follow the component pages below in menu order. Each explains its responsibility, communication partners, exchanged values and completion conditions. Detailed Rust declarations and backend investigations are linked reference material.
3. **E2E flows:** start with [Transaction to durable state](e2e/normal.md), then follow body exchange, native finality, scheduling, certification and recovery cases as needed.

## Components and their connections

| Component | Responsibility | Communication to understand |
|---|---|---|
| [Multimmit](consensus/README.md) | Producer lanes, DA and native consensus | Calls App's `Automaton`; publishes native activity and relay requests to Marshal |
| [Marshal](consensus/block-body.md) | Full-block custody, exchange and ordered delivery | App uses its mailbox; Marshal sends indexed `Update`s and receives durable-application ACKs |
| [App coordination](baton/README.md) | Callback intake, lifecycle, dispatch and retained Update ownership | Connects the component calls; retains delivery tokens until certified durability |
| [TxPool](tx/interfaces.md) | Transaction admission, retention and selection | Receives transactions, supplies proposal batches and reconciles durable outcomes |
| [Alignment](baton/direction.md) | Candidate admission, ordering reflection and next-work selection | `admit`, ordering-only `commit`, and internal `next` over the execution view |
| [Executor](execution/README.md) | Transaction computation and in-memory checkpoints | Receives exact-parent jobs, returns StateChanges, reads cold state and installs durable checkpoints |
| [Storage](execution/storage.md) | Pinned reads, roots, certified persistence and recovery | `read/submit/certify`; retained `Prepared/Committed/Failed` events |
| [App result processing](execution/interfaces.md) | Direct-result signing and peer certificate collection | Combines direct execution evidence with prepared commitments and submits certificates to Storage |

[State sync](execution/state-sync.md) explains certified checkpoint handoff across these existing owners. [P2P](overview/networking.md) identifies the native and App message paths. These responsibilities can share concrete owners and handles; the table does not require one service or public trait per row.

## Before following E2E

Keep three completion boundaries in mind:

- `verify(true)` establishes payload validity and custody; eligible execution is dispatched independently.
- `Alignment::commit(ordered)` records finalized ordering immediately. `Storage::submit` and `certify` success mean accepted work or evidence.
- `StorageEvent::Committed(applied)` establishes certified local durability, supplies Executor's next usable checkpoint and permits ACK of the covered Updates. Executor installation adds no ACK round.

The E2E pages reuse the component names and communication contracts above. They add timing, concurrency, retries and failure paths. The [complete contents](SUMMARY.md) and generated previous/next links follow the same reading order.

## Reference and document status

Use [Communication contracts and type details](overview/interfaces.md) for exact inputs, outputs, trait visibility and invariants, and [Existing Rust signatures](overview/rust-interfaces.md) for source-grounded Commonware declarations. QMDB, pool backend research, baseline comparisons and development checks follow the E2E section as reference material. Markdown is canonical; the site and Rust export are generated from it.

These pages describe a proposed App connected to native source at `6233438985d8249d2b2bc1204191d5d405652288`. App implementation, protected-prefix Baton native integration, E2E execution and performance results remain incomplete. Pre-cut and Baton share the App execution/storage contracts; full Baton additionally needs authenticated native policy adoption, preservation and recovery. Backend, comparator, wire and resource policies remain open. The Bank application model remains outside scope.
