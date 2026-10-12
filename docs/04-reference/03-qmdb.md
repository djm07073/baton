# QMDB branches and persistence

<a id="qmdb-branches-reuse-and-state-application"></a>

**Executor computes in memory; Storage owns QMDB and persistent checkpoints.** Executor produces exact effects and outputs over a supplied read-only base. After [`Storage::submit`](01-communication-contracts.md#proposed-storage-interface), Storage materializes selected changes and emits `Prepared` commitments. Executor verifies matching `f+1` evidence over the prepared state root and exact execution subject, then requests `commit(verified)`; Storage matches prepared material, batches persistence and emits `Committed` with a usable checkpoint. Executor uses `Storage::read` for cold values at that pinned base. A reusable speculative checkpoint need not imply hashing every abandoned attempt.

The current recipe below was checked against Commonware `6233438985d8249d2b2bc1204191d5d405652288`. Earlier `534af0e…` and release comparisons remain [historical reuse evidence](../../assets/review/commonware-reuse-20261004/README.md); they are not the current `DatabaseSet` call sequence.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](01-communication-contracts.md#diagram-notation) for contracts and implementation ownership.

## QMDB state and reuse boundaries

QMDB supplies authenticated database, batch, journal and persistence algorithms, not transaction semantics. Mutable keyed Any maintains an operation journal and latest-key index. Current adds authentication of active operations; its canonical root differs from the operations root used by sync. Choose the result-root scheme explicitly. A storage root alone does not bind an execution range, runtime or completed direct work. [QMDB](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/storage/src/qmdb/mod.rs), [Current root/witness](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/storage/src/qmdb/current/db.rs#L274).

An unmerkleized batch retains pending mutations. Merkleization consumes the draft and produces a sealed batch with root and ancestry; it does not apply that batch canonically. The app associates each retained batch/effects checkpoint with exact base, runtime, ordered input prefix, outputs and direct/imported provenance. Local generations and retention bookkeeping are not shared signature fields.

```mermaid
flowchart TB
    M["Multimmit Marshal"] <-->|"[C] report(Update) / ACK"| A["App canonical owner"]
    A <-->|"[A] execution.execute / reconcile<br/>ExecutionOutcome / StateChanges<br/>execution.install_checkpoint: local adoption"| E["App Executor<br/>execution, checkpoints and f+1 certification"]
    subgraph S["Storage"]
        R["Pinned read views"]
        P["Selected material and root preparation"]
        B["Concrete QMDB batches"]
        W["Match verified subject and material<br/>single canonical writer"]
    end
    A <-->|"[P] Storage::submit(executed, changes)<br/>Ok(()) accepts the handoff"| P
    P -->|"[P] StorageEvent::Prepared"| E
    P -->|"[C] new_batches / fork_batches<br/>[A] materialize StateChanges<br/>[C] merkleize"| B
    B --> W
    E -->|"[P] Storage::commit(verified)"| W
    W <-->|"[C] apply / finalize<br/>Barrier::durable() == true"| D["DatabaseSet / QMDB"]
    W -->|"[P] StorageEvent::Committed(applied)<br/>after batched durability"| A
    E <-->|"[P] Storage::read(view, key)<br/>pinned cold state"| R
    classDef reuse fill:#dbeafe,stroke:#2563eb,color:#172554;
    classDef fresh fill:#dcfce7,stroke:#16a34a,color:#14532d;
    class M,B,D reuse;
    class A,E,R,P,W fresh;
```

[Open full-size diagram](../assets/diagrams/diagram-17.svg)

The subgraph is one application, not a required actor graph. Use concrete `glue::stateful::db` utilities if they fit; do not require `commonware_glue::stateful::Application` or the full Stateful actor. Its merkleized execution lifecycle is not necessary for deferred-root speculation over merged Multimmit input.

<a id="existing-apis-at-the-native-pin-and-indexed-release"></a>

## Current QMDB APIs

| Need | Existing current API | App obligation |
|---|---|---|
| Draft at applied base | `DatabaseSet::new_batches()` | Match actual canonical base/runtime/input position |
| Child of pending sealed parent | `DatabaseSet::fork_batches(&parent)` / `Merkleized::new_batch()` | Retain required ancestors and exact execution identity |
| Selected root | Concrete `Unmerkleized::merkleize()` / sealed `root()` | Deterministic boundary, completed effects and full output/material binding |
| Apply batch | `DatabaseSet::apply(sealed).await` | Single writer, valid ancestry and authorized exact input |
| Start persistence | `DatabaseSet::finalize().await` | Await prior barrier before finalizing again |
| Observe durability | Returned `Barrier::durable().await` | Require `true` and certified state/output/applied-index/evidence linkage before ACK |
| Query applied state | `DatabaseSet::readers()` / `Reader::read()` | Guard current DB access; bind checkpoint/readiness separately |
| Prune | `DatabaseSet::prune(targets)` | Targets already durable, no pending barrier, retention obligations satisfied |

`apply` and `finalize` are separate current calls. A barrier covers checkpoints applied before that finalize call, not later applies. Do not overlap mutation method bodies or prune while durability is pending. Multi-database application does not automatically establish crash-atomic linkage with an app output journal or Marshal's separate ACK cursor. [Current DatabaseSet](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/glue/src/stateful/db/mod.rs#L498), [ManagedDb ownership](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/glue/src/stateful/db/mod.rs#L327).

DatabaseSet has no tuple-wide `merkleize` or single root. Seal concrete component batches and bind the chosen roots/outputs under the app's deterministic result rule. Keyless, compact, Any and Current differ; no variant is adopted by this design.

<a id="give-executor-concrete-branch-access"></a>

<a id="give-app-execution-concrete-branch-access"></a>

## Give Executor a read-only base

Storage retains mutable database and batch handles. Executor receives a read-only view bound to the exact canonical checkpoint and builds its speculative parent views from in-memory effects. A view can read storage through Storage-owned access; it need not copy the whole DB into memory. Canonical query readers alone supply neither a pinned historical snapshot nor a speculative pending-parent view. The concrete versioning/materialization/access-fence implementation remains App work.

Inside Storage, generic `Unmerkleized` exposes merkleization; keyed `get`/`write` belong to selected concrete wrappers. Use these existing handles to materialize selected Executor effects, without transferring canonical mutation authority to execution workers. [Any wrapper](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/glue/src/stateful/db/any.rs#L95).

Batch reads can fall through to the shared DB's **current applied state**, not an immutable snapshot from creation. Storage must preserve valid ancestry during reads, child creation, materialization and hashing, including reads made through Executor's supplied view. Advancing the DB along an actual ancestor can remain valid; applying a sibling can invalidate a retained batch. Validate and use under the same access authority, including all shared/cloned handles. Draft creation and queries must not observe a partially applied database set. Generation checks alone reject stale results but do not prevent invalid reads.

`Shared` is writer-preferring. Do not implement that fence by holding its read guard across `batch.get()` or another operation that reacquires the same cell: a queued writer can deadlock the nested read. [Existing lock contract](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/glue/src/stateful/db/mod.rs#L148).

Keep every still-needed unapplied ancestor alive. A child draft retains its immediate parent strongly, while retained sealed ancestor links can be weak. Keeping only a leaf is insufficient for further branch reads, proofs or child merkleization. Use existing handles, not duplicated QMDB ancestor machinery. Concrete `validate_batch` can reject invalid ancestry before consuming a DB, but does not verify application semantics or prevent later I/O failure. [Ancestor retention](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/storage/src/qmdb/any/batch.rs#L2855), [Current preflight/apply](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/storage/src/qmdb/current/db.rs#L665).

## Defer roots without inventing an unsealed-parent fork

| Path | Existing support | Remaining app work |
|---|---|---|
| Continue one draft | Concrete unsealed reads/writes | Exact effects/outputs and execution context |
| Fork a sealed prefix | Existing child batches | Choose useful sealing boundaries and retain ancestors |
| Branch from an unsealed completed prefix | Independent drafts at a valid applied/sealed anchor | Replay retained exact effects or use an application overlay; an unsealed-parent fork is not supplied |

For rootless AB with children C and D, Executor retains AB's exact base-bound effects and computes C/D using separate in-memory overlays. Storage materializes a selected prefix into a draft at its valid anchor, replaying the exact effects under the deterministic storage rule. This does not require an upstream unsealed-parent fork or grant access to an unavailable old base. If effects or a usable anchor are absent, request reexecution from a valid checkpoint.

Unsealed/staged wrappers can be single-owner values consumed by merkleization. Retain needed effects before consuming the only draft. Staged read locations do not imply computed values are already visible in future reads; preserve read-your-writes semantics in the chosen path. Rootless replay/overlay remains a choice, not an adopted extra generic read-view service.

Prepare a deterministic selected prefix under the chosen operation/batch rule. Identical transaction order and final key values do not automatically yield identical operation histories or roots when nodes choose different speculative sealing boundaries. Writing one key twice in one batch versus sealing between writes can produce different operation histories. Do not sign arbitrary local speculative-batching roots as matching results. Root scheme, normalization and canonical boundaries remain open.

## Manage the branch tree for execution requests

Executor's in-memory tree tracks exact execution prefixes above a Storage-supplied canonical checkpoint. One producer block after different prior inputs has different execution identity, even when its header/body digest is unchanged. The execution parent is not `Context.parent`. A child may run only after its actual predecessor checkpoint is valid.

```mermaid
flowchart LR
    S["S0: canonical checkpoint"] -->|"[A] execution.execute(job)<br/>A @ S0"| A["Checkpoint A"]
    A -->|"[A] execution.execute(job)<br/>B @ A"| AB["Checkpoint AB"]
    AB -->|"[A] execution.execute(job)<br/>C @ AB"| ABC["Checkpoint ABC"]
    AB -->|"[A] execution.execute(job)<br/>D @ AB"| ABD["Checkpoint ABD"]
    S -->|"[A] execution.execute(job)<br/>X @ S0"| X["Checkpoint X"]
    X -->|"[A] execution.execute(job)<br/>A @ X"| XA["Checkpoint XA: different execution"]
    classDef canonical fill:#dbeafe,stroke:#2563eb,color:#172554;
    classDef reuse fill:#dcfce7,stroke:#16a34a,color:#14532d;
    classDef pending fill:#ffedd5,stroke:#ea580c,color:#431407;
    class S canonical;
    class A,AB reuse;
    class ABC,ABD,X,XA pending;
```

[Open full-size diagram](../assets/diagrams/diagram-18.svg)

This is a checkpoint derivation graph. Each **[A]** `execution.execute(job)` edge shows the job's `block @ parent` context, followed by accepted `app.finish_job(job_id, outcome)`. It describes completed valid execution, not a producer-header link or another public method.

A valid requested path A→B→D can reuse completed AB from the same S0/runtime and run D. A after X cannot reuse A from S0 merely by block identity. Ordinary late arrivals preserve started order; precedence for a conflicting advisory direction remains open. Canonical Updates determine which exact path must be reconciled.

After Storage supplies a durable AB checkpoint, Executor installs it and retains C/D descendants only if exact state/runtime/read dependencies remain valid. Storage separately checks actual ancestry of any retained storage batches. After installing ABD, Executor retires competing C and X work and skips every input already covered by ABD. Executor releases memory after safe worker/reference release; Storage controls disk pruning and checkpoint-view retention for queries, recovery, result serving and sync. Canceling an awaiting future may leave CPU work running; protect state-view reads separately from eventual resource release.

## Commit a branch to canonical state

| Step on Marshal Update | Required condition |
|---|---|
| Admit ordered index/block/ACK | Preserve Marshal order; do not substitute a local scheduler order |
| Reconcile from preceding canonical state | Exact input, runtime and execution ancestry match |
| Reuse/complete/repair or import | Completed valid direct effects, or fully verified applicable imported material |
| Prepare selected boundary | Full exact-prefix/root/output identity; no slicing an unrelated sealed batch |
| Certify before writer mutation | Verify `f+1` distinct eligible matching signatures on that whole input/base/runtime/result, for direct work and imports |
| Apply and persist through one writer | App state, outputs, applied identity, certificate evidence and provenance recover consistently |
| Supply usable canonical checkpoint | Storage emits `Committed` with exact identity and pinned state view; App owner forwards it to Executor to skip covered work |
| ACK Update | Matching certified durable application established; Marshal syncs its cursor independently |
| Maintain pool and branch retention | Reconcile durable outcomes; retain still-required references |

Applying a sealed leaf also applies its unapplied ancestors, so certify and authorize the entire exact prefix it carries. If ABC was speculatively executed but only AB is ordered, prepare and certify AB before applying exactly AB. A terminal certificate for ABC cannot authorize an AB checkpoint unless its signed subject explicitly covers that intermediate boundary. If only a sealed ABC batch exists, applying it and hiding C is invalid; prepare AB from retained exact effects or reexecute. A newly prepared AB commitment may differ from ABC's original parent commitment, so old C reuse requires actual ancestry checks as well as logical input checks.

Storage retains submitted changes independently of flush timing. One canonical writer serves direct and imported results after certificate verification. Physical batching may group successive certified units, but must preserve each signed unit's deterministic operation history, root and recoverable evidence; idle tails and a full delivery window must trigger bounded progress. `submit` acceptance is not durability or permission to prune Executor checkpoints. Executor verifies original certificate evidence before commit; Storage matches that verified authority to its material and actual predecessor before mutation; missing or mismatched certification leaves prepared work pending. Alignment records ordering earlier and supplies no state-commit approval. Advisory cancellation cannot interrupt a by-value DB mutation and turn the lost handle into an ordinary retry. Mutable storage failure is fatal for that instance; recover authoritative durable state rather than continue using it. A readable applied DB with pending flush is not durable completion. A closed/failed barrier is not success or proof of zero disk writes.

Reuse existing QMDB, Journal and Metadata mechanisms for app recovery linkage. An atomic Metadata update covers its own store; it does not atomically commit QMDB and another journal. Record enough exact identity and certificate evidence to recover certified state/output/applied-position/provenance consistently before ACK. A durable evidence reference must remain resolvable under pruning. Marshal separately owns its durable delivery cursor; a crash between those stores can redeliver an already applied block, which the app validates and ACKs idempotently. Missing evidence cannot establish applied readiness. No second generic cursor engine or public CommitResult type is required.

An applied checkpoint may survive a crash before durability was observed. Existing `ManagedDb::init(expected)` and `DatabaseSet::init(..., Some(targets))` reopen selected checkpoints and durably discard later state. App must select matching database targets, outputs, certificate evidence and provenance under its recovery rule; opening each DB's latest checkpoint or reading Marshal's cursor alone does not choose a consistent App checkpoint. That multi-store selection/linkage remains implementation work. [Existing recovery contract](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/glue/src/stateful/db/mod.rs#L334).

`readers()` returns handles to the live databases, not pinned historical versions. A held `Reader::read()` guard stabilizes that DB during a query; separate calls or tuple-component guards do not by themselves establish one atomic App checkpoint. Bind value, proof, root and applied identity under the same valid access authority. [Reader implementation](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/glue/src/stateful/db/mod.rs#L210).

Current value/absence proofs can serve guarded app queries, but an operation-inclusion proof does not prove current absence. These proofs do not supply the full execution certificate or establish durability. Query scope and historical retention remain open.

For active imports, use the same writer/fences and verify both certificate and material before stopping direct work. [Certified state sync](../02-components/09-result-processing.md#state-sync-from-certified-execution-results) explains why a progress notification or Marshal floor alone cannot establish application-state readiness.
