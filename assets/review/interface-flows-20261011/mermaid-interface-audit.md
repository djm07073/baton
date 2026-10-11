# Mermaid-to-interface audit — 2026-10-11

Scope: all 19 current inline Mermaid diagrams, their generated sources and the current interface specification. This is a documentation contract audit, not compilation of an SDK, a runtime test or a native-protocol proof. The accompanying JSON pins each reviewed diagram's source hash.

## Result

Every named call/event now maps to a documented contract or an explicitly marked unresolved obligation. This does **not** mean every arrow is a settled public Rust interface:

- **[C]** uses existing Commonware methods; the native reference and local source establish their meaning.
- **[P]** uses proposed `Alignment`, internal `AlignmentPolicy`, `Storage` and `StorageEvent` contracts.
- **[A]** uses documented private App helpers or describes internal behavior. It does not require another SDK trait.
- **[X]** identifies the four open native-policy obligations. Concrete callable signatures are not settled.
- **wire** identifies existing native artifacts or logical App messages whose concrete codecs remain open.
- Unlabelled native actor edges and the checkpoint derivation graph describe dependencies/state transitions rather than additional RPCs.

See the [notation contract](../../../docs/overview/interfaces.md#diagram-notation) and [wire-message catalog](../../../docs/overview/networking.md#messages-used-in-diagrams). Overview diagrams intentionally group methods; detailed sequences and the interface tables provide their arguments and completion conditions.

## Corrections made during this audit

| Finding | Correction |
|---|---|
| Storage events alternated between qualified names and bare `Committed` / `Failed` labels | Detailed sequences now name `StorageEvent` variants; overview groups explicitly identify the enum. Failure payload is `StorageFailure`, labelled `failure`. |
| Storage method success looked like a separate `Accepted` return type | Sequence returns now show `Ok(())`; read returns show `Ok(Some(value))`, `Ok(None)` or `Err(StorageError)`. The event is distinct from method completion. |
| Recovery table said `AppliedCheckpoint` return, while diagrams sent a `Committed` event to a different consumer | Private recovery emits retained `StorageEvent::Committed(recovered: DurableApplied)`; Executor receives `recovered.checkpoint`. App open creates handles; Storage separately supplies the recovered base. |
| PreCut overview appeared to make the policy implementation dispatch Executor work | Callback/dispatch owner calls the policy traits and executes/reconciles work; `PreCutAlignment` only supplies the documented policy behavior. |
| Grouped Executor arrows suggested execute, reconcile and checkpoint installation all return StateChanges | Execute/reconcile show `ExecutionOutcome` / `StateChanges`; installation is local adoption, with no StateChanges return. |
| `ScheduleChanged` resembled a prescribed event type | Replaced with the specified generic coalesced dispatch wakeup, with no payload/type requirement. |
| Import verification returned PreparedImport without making retained material ownership explicit | The helper retains material in Storage under the exact target before returning; `certify` resolves that retained candidate. The helper remains a private integration contract. |
| Startup omitted runtime from Engine open and folded network start into prose | `Engine::open(runtime, config)` and separate `network.start()` now match the documented existing calls. |
| Checkpoint edges used an `execute(block @ parent)` shorthand that looked like another method | Labels now use `execution.execute(job)` and separately show the block/parent context. |
| App wire labels were spread across flow prose | Added a single catalog covering every logical message family and distinguishing unsettled App schemas from existing native artifacts. |

## Coverage of all diagrams

| Diagram | Subject | Specification mapping and qualification |
|---|---|---|
| 01 | App composition | [Native connections](../../../docs/overview/interfaces.md#existing-interfaces-to-connect), [Alignment](../../../docs/overview/interfaces.md#proposed-alignment-interface), [Storage](../../../docs/overview/interfaces.md#proposed-storage-interface); grouped method/return labels |
| 02 | Transaction through durable state | [Pool/execution helpers](../../../docs/overview/interfaces.md#private-pool-and-execution-calls), [canonical handlers](../../../docs/overview/interfaces.md#private-canonical-and-state-calls), Storage requests/events and native callbacks |
| 03 | Transaction intake and maintenance | [Pool contract](../../../docs/tx/interfaces.md) and private pool/publication helpers; Admission variants are logical values, not released Rust enums |
| 04 | Local body creation and relay | [Existing body interfaces](../../../docs/overview/interfaces.md#existing-interfaces-to-connect) and [body contract](../../../docs/consensus/block-body.md); native sender/receiver completions |
| 05 | Verification and body retrieval | [Callback behavior](../../../docs/baton/interfaces.md#automatonverify-validity-custody-and-scheduling), Mailbox calls and private admission/dispatch helpers |
| 06 | Report window | [Baton rules](../../../docs/baton/direction.md), [messages](../../../docs/overview/networking.md#messages-used-in-diagrams); `leader_context` remains [X], private report processing is behavior |
| 07 | Direction and dispatch | [Internal policy](../../../docs/overview/interfaces.md#internal-selection-through-alignmentpolicy), private dispatch/execution helpers and coalesced wakeup; policy does not claim resources or execute |
| 08 | Ordered input and state completion | [Canonical handlers](../../../docs/overview/interfaces.md#private-canonical-and-state-calls), Storage submit/certify/events and retained Update ACK |
| 09 | Cold reads, changes and QMDB | [Storage](../../../docs/overview/interfaces.md#proposed-storage-interface), [QMDB API table](../../../docs/execution/qmdb.md#current-qmdb-apis) and concrete execution helpers |
| 10 | Restart/redelivery | [Recovery/lifecycle helpers](../../../docs/overview/interfaces.md#private-result-import-and-lifecycle-calls), recovery event contract and native delivery/cursor behavior |
| 11 | Result certification | Storage submit/Prepared/certify, [result helpers](../../../docs/overview/interfaces.md#private-result-import-and-lifecycle-calls) and logical result messages |
| 12 | Native policy adoption | [Four native obligations](../../../docs/overview/interfaces.md#baton-behavior-and-native-extension); offer/adopt/validate remain [X], future policy encoding is not an existing wire schema |
| 13 | Startup assembly | [Existing interfaces](../../../docs/overview/interfaces.md#existing-interfaces-to-connect), [startup sequence](../../../docs/e2e/recovery.md#startup-prepare-custody-before-native-recovery) and private assembly/recovery contract |
| 14 | Certified state import | [Private import contract](../../../docs/overview/interfaces.md#private-result-import-and-lifecycle-calls), [state-sync checks](../../../docs/execution/state-sync.md), certify/Committed and checkpoint installation |
| 15 | Producer/DA/finality | [Native protocol](../../../docs/consensus/README.md) and existing Automaton/Reporter; named wire artifacts retain native codecs and internal transitions are not App methods |
| 16 | Native actors | [Native source layout](../../../docs/consensus/README.md#native-actors-and-source-layout); labelled crossings are existing callbacks, other edges are internal dependencies |
| 17 | Storage ownership | Storage requests/events, concrete execution helpers and [QMDB operations](../../../docs/execution/qmdb.md#current-qmdb-apis); read views and root preparation are distinct Storage responsibilities |
| 18 | Checkpoint derivation | [Execution helper](../../../docs/overview/interfaces.md#private-pool-and-execution-calls) and [branch tree](../../../docs/execution/qmdb.md#manage-the-branch-tree-for-execution-requests); Job contains the depicted exact parent |
| 19 | PreCut composition | Shared Alignment/AlignmentPolicy and Storage interfaces, private App dispatch/execution/pool helpers, native callbacks and logical result/material traffic |

## Native-source checks

Existing methods were compared with local source: Marshal mailbox custody/fetch/subscribe calls; Service relay/start; Engine open/start; example network/buffer/resolver assembly; DatabaseSet new/fork/apply/finalize, concrete merkleization and Barrier durability. Their native pin remains `6233438985d8249d2b2bc1204191d5d405652288`; no native Rust file was changed.

The App helper and value tables define logical completion/error contracts, not final implementation signatures. In particular the generic public Storage trait does not prescribe import/open methods: the diagrams identify these as concrete [A] assembly/import integration. The four [X] obligations and App wire codecs still require design before implementation. This audit does not promote them into settled interfaces.

## Verification

The machine checks validate diagram/source hashes, local document/anchor targets, rendered assets and site navigation. Manual review checks caller/owner direction, request versus event meaning, input/output identity and correspondence to the specification. Neither kind establishes runtime correctness.
