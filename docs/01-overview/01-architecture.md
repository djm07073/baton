# Architecture

**Consensus orders blocks; Alignment schedules; App Executor computes; Storage stores and supplies canonical checkpoints.** Alignment, Executor and Storage are separate responsibilities inside one App. Existing Marshal supplies custody, body exchange and ordered delivery. Alignment layer implements existing native callbacks and selects the proposed common Alignment implementation: PreCut or Baton.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [communication reference](../04-reference/01-communication-contracts.md#diagram-notation) for call/event notation.

## Application composition

```mermaid
flowchart TB
    N["Consensus<br/>Multimmit Engine"] -->|"[C] Automaton::propose / verify"| M
    N -->|"[C] Relay::broadcast<br/>Reporter::report(Activity)"| R["Existing Multimmit Marshal"]
    M <-->|"[C] App calls stage / subscribe / fetch / ACK<br/>Marshal calls report(Update)"| R
    subgraph APP["Application"]
        subgraph MW["Alignment layer"]
            M["Callback handles and Update owner"]
            M <-->|"[P] Alignment::admit / commit(ordered)<br/>internal AlignmentPolicy::next"| S["PreCutAlignment<br/>or BatonAlignment<br/>pub(crate) implementations"]
        end
        P["App pool and payload code"]
        E["App Executor<br/>execution, checkpoints and certification"]
        C["Storage<br/>cold reads, roots and persistence"]
        M -->|"[A] pool.select(ctx, limits)"| P
        M <-->|"[A] execution.execute / reconcile<br/>ExecutionOutcome / StateChanges"| E
        M -->|"[P] Storage::submit(executed, changes)"| C
        C -->|"[P] StorageEvent::Prepared"| E
        E -->|"[P] Storage::commit(verified)"| C
        C -->|"[P] StorageEvent::Committed"| M
        E <-->|"[P] Storage::read(view, key)<br/>pinned cold state"| C
        C -->|"[A] via App owner<br/>execution.install_checkpoint"| E
    end
    C -->|"[C] apply / finalize / Barrier::durable"| Q["QMDB / DatabaseSet"]
    classDef reuse fill:#dbeafe,stroke:#2563eb,color:#172554;
    classDef app fill:#dcfce7,stroke:#16a34a,color:#14532d;
    class N,R,Q reuse;
    class M,S,P,E,C app;
```

[Open full-size diagram](../assets/diagrams/diagram-01.svg)

Blue boxes are existing components. Green boxes separate App pool, Alignment, Executor and Storage responsibilities; they can share one owner and concrete handles. The [interface map](../04-reference/01-communication-contracts.md#proposed-alignment-interface) separates public `Alignment` calls from crate-internal `AlignmentPolicy::next`. Both alignment implementations share those contracts and use the separate public [`Storage`](../04-reference/01-communication-contracts.md#proposed-storage-interface) contract for prepared roots and Executor-authorized durable application. The diagram does not prescribe one actor per box. Executor workers compute in memory; Storage owns storage I/O. Both keep slow work off short callback handlers. Commonware runtime and authenticated P2P supply task and transport mechanisms.

<a id="four-layers"></a>

<a id="responsibilities-and-existing-entry-points"></a>

## Responsibilities and communication

| Component | Responsibility | Communication |
|---|---|---|
| [Multimmit](../02-components/01-multimmit.md) | Producer lanes, DA, views and native finality | Calls App Automaton; sends Activity and relay requests to Marshal |
| [Marshal](../02-components/02-marshal-custody.md) | Full blocks, native history, backfill and ordered delivery | Custody mailbox calls; indexed Update and application ACK |
| [App coordination](../02-components/04-app-coordination.md) | Callback intake, lifecycle, claimed dispatch and Update tokens | Connects pool, policy, execution, result processing and Storage |
| [TxPool](../02-components/05-tx-pool.md) | Admission, retained candidates and bounded selection | Transactions in; proposal batches out; durable outcomes for maintenance |
| [Alignment](../02-components/06-alignment.md) | Eligible pending work and finalized-order reflection | `admit/commit`; internal `next` selects an intent from the execution view |
| [Executor](../02-components/07-executor.md) | Transaction computation, checkpoints and result certification | Exact-parent jobs, StateChanges, prepared roots, verified commit requests and checkpoint installation |
| [Storage](../02-components/08-storage.md) | Roots, material checks, persistence and recovery | `read/submit/commit`; `Prepared/Committed/Failed` events |
| [Result certification and state sync](../02-components/09-result-processing.md) | Executor verifies certificates; Storage persists matching direct or imported material | Verified Storage commit → durable checkpoint → Executor installation and App ACK |

The current source is [Commonware checkout 6233438](https://github.com/0xEyrie/monorepo/tree/6233438985d8249d2b2bc1204191d5d405652288). Current Multimmit Marshal replaces the older proposed application body service and ordinary Baton-owned order-delivery machinery. Historical research pinned to `534af0e` remains useful for its stated questions, but does not describe this new assembly.

<a id="inputs-and-outputs-between-layers"></a>

## Inputs and outputs between components

`Automaton` connects App's pool and payload validation to Multimmit; Marshal connects block custody and canonical delivery to App. The [App callback flows](../02-components/04-app-coordination.md) define staging, all `verify` origins, scheduler admission, asynchronous application and ACK handling.

## Data, control, and canonical paths

| Path | Flow | Established fact |
|---|---|---|
| Payload | Pool → propose → Marshal custody and relay | Exact block bytes retained and disseminated |
| Native consensus | Header/DA/view/certificate planes → native machine | Authenticated lane and ordering facts |
| Speculation | Usable candidate → App scheduler → exact-parent execution | Completed local effects for a particular path |
| Baton control | Intended-order report → bounded selection → advisory direction | Execution intention; no finality or result proof |
| Canonical delivery | Native history → Marshal → indexed Update → Alignment::commit(ordered) | Continuous finalized ordering, independent of execution progress |
| Application durability | Executor effects → Storage preparation → Executor certification → Storage Committed event → ACK | Recoverable certified state, outputs, certificate evidence and applied position |
| Execution catch-up | Storage checkpoint → App owner → Executor installation | Skip the exact covered input and resume from its usable state view |
| Result certification | Peer Apps exchange signatures over the exact statement | `f+1` matching distinct signatures plus irrevocable order/base establish the research result endpoint |

Body custody, speculation, native finality, result certification and local durable application are separate facts. Body/header ancestry does not supply the cross-lane execution state parent. Only matching exact input/base/runtime permits reuse.

## Workers and decision authority

Use short App handlers to admit work and update metadata; run transaction computation, report-candidate evaluation and storage operations through appropriate Commonware tasks. Bound queues and retained bytes. Canonical deliveries need a non-dropping inbox matched to Marshal's ordinary delivery window, with old/new window overlap explicitly handled during floor/import transitions; best-effort speculation may defer or forgo work when its own budget is exhausted.

The scheduler selects pending jobs, while shared execution state owns checkpoints and stale-worker fences. The canonical writer serializes state mutation, including certified imports. Reports and advisory direction cannot gate canonical delivery, result certification, state sync or ACK. Physical pruning waits for reference/retention obligations even after logical conflicting-branch removal.

Root preparation remains separate from transaction computation. Reuse QMDB directly instead of requiring every speculative attempt to return a Merkleized result through the full glue actor. Root-deferred branching still needs a concrete application integration; existing sealed-parent fork APIs do not solve it automatically.

## Pre-cut and full Baton

Pre-cut is an independent scheduler configuration with no Baton reports/direction or policy hook. Both modes preserve started local work and sort only eligible pending candidates; both reconcile against the same authoritative Marshal Update stream.

Baton additionally needs actual leader context and authenticated native policy adoption to preserve a chosen prefix in canonical order. Existing Automaton methods are producer-payload callbacks, and current Marshal interprets the native fixed order. An app-only scheduler can guide speculation; full Baton requires specific native/Marshal changes and proofs. Do not reorder already delivered Updates to simulate that missing integration. [Direction and the native gap](../02-components/06-alignment.md#existing-callbacks-versus-the-native-policy-gap) states the outstanding contract.
