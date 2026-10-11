# Communication contracts and type details

<a id="interface-map-what-to-implement-and-connect"></a>

**Use this reference to check who calls whom, what crosses the connection and what completion establishes.** Start with the [component reading path](../README.md#components-and-their-connections) for responsibility summaries, then use the contracts here beside the E2E diagrams. The [Rust reference](rust-interfaces.md) contains existing Commonware declarations.

Proposed App traits define shared communication contracts. Concrete helper names describe private App behavior; they do not require another public trait. PreCut and Baton share `Alignment` and the crate-internal `AlignmentPolicy`; Storage has its separate request/event contract. These proposals are not an implemented or released SDK.

Using a supplied alignment implementation does not require reimplementing its traits. Adding a policy with the current `pub(crate)` contract happens inside the SDK crate; third-party policy plugins would need an explicitly public extension contract. Each method below specifies values, state changes, completion and failure behavior while leaving private decomposition to its owner.

<a id="consensus-alignment-and-app-executor"></a>

## Consensus and application responsibilities

| Responsibility | Owns | Calls / completion |
|---|---|---|
| **Consensus** | Multimmit producer lanes, DA, proposals, votes and native finality | Calls the existing Automaton; publishes native activity to Marshal |
| **Alignment layer, inside App** | Automaton/Update callback handles, retained delivery tokens, lifecycle, PreCut or Baton scheduling and dispatch coordination | Requests payload work and execution; returns verify after validity/custody; ACKs exact delivery after durable application |
| **App pool and payload code** | Transaction admission/selection and block payload checks | Supplies propose/verify without waiting for execution |
| **App Executor, inside App** | Transaction execution, in-memory effects/overlays and speculative checkpoints | Reads pinned cold state and submits exact ordered changes; installs committed checkpoints and skips covered input |
| **Storage, inside App** | Storage handles, root/material preparation, certificate validation, canonical persistence, recovery, import and durable checkpoint retention | Accepts submissions independently of persistence; emits `Prepared`, `Committed(DurableApplied)` and failures; supplies pinned read views |
| **App result processing** | Direct-evidence signing and peer signature exchange/collection | Combines Executor evidence with Storage-prepared commitments; calls `Storage::certify` with collected evidence |
| **Existing Marshal, reused alongside these roles** | Full-block custody/relay/backfill, native history interpretation, ordered delivery and cursor | Provides mailbox/Relay interfaces and reports indexed Updates to Alignment layer |

Alignment, App Executor and Storage describe separate responsibilities within one App. They can share an owner and concrete handles; the diagrams do not require a process or actor per role. The native type spelled **`AppExecutor`** runs Commonware's local Automaton callback jobs. It is different from the **App Executor** transaction-execution role designed here. The pool remains App-owned, alongside these modules.

Alignment layer implements the existing `Automaton` and `Reporter<Update>` contracts on its callback handles. The proposed `Alignment` below defines external calls; `AlignmentPolicy` defines the internal selection contract shared by its two implementations. An additional `Executor` trait is useful if App execution implementations must be independently replaceable; the current private execution calls describe that contract without prescribing another public trait. Custody and ordered delivery continue to use Marshal directly.

## Proposed Alignment interface

**These are App-side design proposals, not existing Commonware traits.** External callers admit candidates and report finalized ordering through `Alignment`. The Alignment layer's internal dispatch loop selects work through `AlignmentPolicy`. PreCut and Baton implement both contracts, allowing the same dispatch loop to use either policy. Storage separately owns certificate verification and canonical application.

```rust
pub trait Alignment {
    fn admit(&mut self, candidate: Candidate) -> CandidateAdmission;
    fn commit(&mut self, ordered: &OrderedView);
}

pub(crate) trait AlignmentPolicy: Alignment {
    fn next(&self, view: &SchedulingView) -> Option<JobIntent>;
}

pub(crate) struct PreCutAlignment { /* ... */ }
pub(crate) struct BatonAlignment { /* ... */ }
```

`AlignmentPolicy: Alignment` requires its implementations to implement `Alignment` as well. Implement both traits separately for each concrete type. Internal dispatch can use a generic bound `P: AlignmentPolicy`; external callers only need `Alignment`. This adds an internal implementation contract, not another runtime layer or actor. Existing `Automaton` and `Reporter<Update>` remain the native entry points.

| Rust declaration | Intended caller / use |
|---|---|
| `pub trait Alignment` | External calls into alignment: candidate admission and finalized-order notification |
| `pub(crate) trait AlignmentPolicy: Alignment` | Internal dispatch shared by PreCut and Baton: choose the next execution intent |
| `pub(crate) struct PreCutAlignment`, `BatonAlignment` | Concrete implementations assembled inside the App crate |

Rust visibility applies to crates and module paths, not conceptual roles. `pub(crate)` permits access throughout the crate; use a private trait or, from a descendant module, `pub(in crate::alignment)` to restrict access to the Alignment module. A cross-crate `Alignment` contract also needs a reachable public module or re-export and public argument/result types (`Candidate`, `CandidateAdmission`, `OrderedView`). `SchedulingView` and `JobIntent` can remain crate-visible with `AlignmentPolicy`. Trait methods inherit their trait's visibility: `next` cannot be hidden by writing `pub(crate) fn` inside a public trait. The names and fields here are a design sketch, not a compilable native export.

### External calls through Alignment

| Operation | Input / result | Contract |
|---|---|---|
| `admit` | Qualified candidate → admitted, already known, deferred or ineligible | Retain candidate metadata once under bounded resources. No execution, body fetch or custody I/O. Alignment layer checks the current role/lifecycle/authentication facts before admission. Deferred speculation does not invalidate a valid verify verdict. |
| `commit` | Exact continuous finalized-order view from retained Marshal Updates | Record immutable ordering and reconcile speculative priority with it. Idempotent on repeated exact input; never infer order across a gap or conflict. No execution, `f+1` wait, storage I/O or ACK. |

`OrderedView` identifies the exact continuous finalized input prefix: ordering context, output indices and block identities, with sufficient linkage to the preceding known prefix. It comes from the retained local Marshal stream or an authenticated native floor/recovery prefix, not an advisory direction or peer assertion. It says nothing about execution completion or state durability. The App owner forwards new ordered progress promptly to `Alignment::commit(ordered)` and independently to canonical execution. Remove a candidate from speculative selection only once canonical intake retains the work; keep required block/checkpoint references until execution and retention rules permit release.

The storage module implements [`Storage`](#proposed-storage-interface) for direct and imported results. It verifies `f+1` distinct eligible signatures on the exact ordered input/base/runtime/result before the canonical writer applies it. Only certified durable application permits Marshal ACK. `Alignment::commit` has already handled ordering and is not called again to signal state application.

### Internal selection through AlignmentPolicy

| Operation | Input / result | Contract |
|---|---|---|
| `next` | Current immutable execution/resource view → block plus exact parent/context, or no intent | Select only eligible unstarted work under the policy, preserving valid started order `F`. The parent must be completed and usable. This call does not claim the job, reserve resources or start execution. |

`SchedulingView` is a read-only projection of authoritative App state: known finalized-order frontier, certified durable state base/runtime, valid completed/current path `F`, exact ready checkpoints, claimed/running attempts, current lifecycle/generation and available resources. The ordering frontier may be ahead of the applied state base; recording it does not make a checkpoint usable. This is not a second execution tree. `JobIntent` identifies a block, its exact execution parent and selection context. These names specify required information, not final Rust fields or new wire types.

Alignment layer handles candidate admission, execution completion and resource availability by asking `AlignmentPolicy::next` again internally. Between selection and launch it rechecks the intent against current state, atomically claims the attempt and reserves resources on the App owner, then records the updated execution view before launching. Another `next` call therefore excludes claimed work. If claim/recheck fails, refresh the view without launching; `next` changed no policy state. On failed launch or worker completion, settle the attempt through the shared lifecycle before offering capacity again. Dropping a waiter does not prove safe termination.

```text
successful verify:
    establish payload validity and durable custody
    refresh candidate eligibility on the Alignment layer owner
    alignment.admit(candidate) if currently eligible
    send true; wake dispatch without awaiting execution

internal dispatch after candidate / completion / capacity change:
    intent = AlignmentPolicy::next(alignment, current scheduling view)
    if no intent: return
    job = recheck and claim intent, reserving resources on the App owner
    if claim failed: refresh view and return without launch
    publish claimed state before launching App Executor.execute(job)
    accept completion once under exact context; settle resources safely
    wake dispatch with the updated view

on authoritative ordered Update:
    retain the full indexed block and its ACK token
    reflect new continuous ordering with alignment.commit(ordered view)
    return Feedback promptly, canonical work continues independently

App canonical owner, Executor and Storage:
    reuse/execute the exact ordered range, or choose a verified peer import
    submit(executed, StateChanges) transfers direct changes, not durability
    continue valid in-memory execution subject to bounded resources

on StorageEvent::Prepared:
    App result processing signs matching direct work and collects evidence
    Storage::certify(certificate) admits evidence, not durability

on StorageEvent::Committed(applied):
    publish exact durable identity and retain applied.checkpoint for Executor
    install_checkpoint skips covered input and retires memory work safely
    wake dispatch with the updated execution view
    acknowledge matching retained Updates

on Storage failure:
    no ACK; retain input and resolve the cause or recover the failed instance
```

`app.admit_candidate` and `app.dispatch_ready` in detailed diagrams are handlers around `Alignment::admit` and internal `AlignmentPolicy::next`. Retained ordering reaches `Alignment::commit`; later `app.publish_applied` updates execution state and wakes internal dispatch without another Alignment callback. Neither trait needs a duplicate execution-tree maintenance hook. Both concrete implementations stay inside the App crate. Neither supplies native policy adoption or reorders Marshal Updates.

`BatonAlignment` owns report collection, deadlines and direction selection internally. Its private methods and task decomposition are left to its implementation. When that processing changes a scheduling choice, wake the shared dispatch loop so it asks `AlignmentPolicy::next` again. A generic coalesced notification through an existing channel is sufficient; its name and mechanism are not another SDK contract. PreCut uses both common traits without Baton protocol machinery.

## Proposed Storage interface

`Storage` owns checkpoint-bound reads, accepted execution changes, result-root preparation, certification checks, persistence, imports and recovery. Executor owns transaction computation and in-memory checkpoints. Both PreCut and Baton use this same interface. Storage hides when and how changes become database batches or durable writes.

**Executor hands over `StateChanges` plus the exact executed range; it does not construct a storage batch, root or certificate.** The App owner validates the range against retained consensus ordering and the accepted execution attempt. It can forward the call without introducing another public trait. Submission is separate from certified durable completion.

```rust
use core::future::Future;

pub trait Storage: Send + Sync {
    /// Reads a key at the supplied pinned state view.
    fn read(
        &self,
        view: &StateView,
        key: StateKey,
    ) -> impl Future<Output = Result<Option<StateValue>, StorageError>> + Send;

    /// Retains changes for an exact executed range without promising durability.
    fn submit(
        &self,
        executed: ExecutedRange,
        changes: StateChanges,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    /// Admits result evidence for verification and eventual durable application.
    fn certify(
        &self,
        certificate: ExecutionCertificate,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;
}

pub enum StorageEvent {
    Prepared(PreparedState),
    Committed(DurableApplied),
    Failed(StorageFailure),
}
```

This is a proposed App SDK interface, not an existing Commonware trait or implemented Rust export. The argument, event and error types specify logical contracts below; concrete fields, codecs and backend types remain open and must be publicly reachable with the trait. Assembly wires a retained, bounded event receiver using existing runtime channels or reporter machinery. No new observer trait is required. Shared handles permit reads and submissions while the implementation serializes canonical mutation; `&self` does not grant concurrent database writers.

| Direction / operation | Meaning and completion |
|---|---|
| Executor → `read(view, key)` | On an in-memory miss, read the exact pinned base. `None` means verified absence at that view; unavailable/pruned views and I/O failure are errors, never absence or fallback to the newest state. |
| Executor → `submit(executed, changes)` | Transfer a complete, exact-parent executed range within known continuous finalized ordering. `Ok(())` means bounded retention accepted, not a root, certificate, durable state or permission to ACK/prune checkpoints. |
| App result processing → `certify(certificate)` | Supply original matching result evidence. `Ok(())` means accepted for processing, not durable completion. Storage independently verifies eligible distinct `f+1` signatures, subject, material and order before application. |
| Storage → `Prepared(prepared)` | Commitments for an exact accepted signing unit are ready. App result handling may sign only with matching retained direct-execution evidence, then collect peer signatures. |
| Storage → `Committed(applied)` | The exact certified prefix and linked state/output/certificate/provenance are recoverably durable. `applied.checkpoint` is usable for Executor installation and covered Update ACKs. |
| Storage → `Failed(failure)` | Identify the affected unit or storage instance and the error. No durable completion is implied. A mutable-store/durability failure stops that instance until recovery; invalid evidence leaves valid retained work pending. |

### Cold state reads

A job reads its current in-memory writes first, then its completed in-memory parent overlays, then `Storage::read(view, key)` at the installed base. A tombstone is a known deletion and must not fall through to an older stored value. Every cold read in that job uses the same exact base/runtime view; advancing storage must not silently change it. The job's speculative ancestors remain Executor-owned overlays, not implicit Storage history.

`StateView` is a retained read-only capability supplied in `AppliedCheckpoint`. Storage pins the required material or provides equivalent versioning/access fencing until readers release it. Existing live QMDB readers are not automatically snapshots. Executor receives no mutable database handle and need not load the entire database into memory. A cold read can await I/O for that job; unrelated execution and consensus callbacks continue independently.

### Ordered progress submission and batched persistence

`ExecutedRange` binds exact continuous ordered inputs and output indices, predecessor state/prefix, runtime and accepted local execution provenance. A numeric high-water mark alone is insufficient. The App owner validates that this work matches its authoritative ordering and live attempt before submission. If Executor has speculated ABC but only AB is ordered, submit AB's retained changes and keep C in memory. Never include C merely because its computation finished.

Assembly supplies Storage with read-only access to the App's authoritative retained ordering context; a peer certificate or caller-supplied frontier cannot replace it. Submission accepts one complete bounded signing unit; its agreed boundaries and changes representation must permit the chosen exact prefix. A later unit may depend on an earlier accepted, still-uncommitted unit. Storage retains that exact predecessor chain, prepares roots in order and defers canonical mutation until every required predecessor and certificate is valid. Executor can continue on its in-memory checkpoints without awaiting each storage commit. A conflicting or stale chain is rejected; overlapping retries do not double-apply changes. Exact duplicate submissions are idempotent; a replay after durable commit re-emits the matching completion.

Accepted changes are owned/retained by Storage, but acceptance may be memory-only. Before acceptance, cancellation or backpressure leaves the caller responsible for retryable changes. After acceptance, dropping a caller does not cancel retained work. On crash, only the recovered certified durable prefix is authoritative; replay/reexecution can reconstruct later accepted work. Executor may release a transferred changes buffer under the ownership contract, but keeps memory/checkpoint data still needed for reads, descendants or replay until durable installation and reference release.

Storage prepares selected signing-unit commitments and emits `Prepared` before certification and application. App result processing gathers signatures and calls `certify`; certificate-only input cannot execute transactions or manufacture missing material. A verified peer import supplies matching material through Storage's existing import path and shares the same writer and `Committed` event. Imported work never generates an own direct-execution signature.

Storage chooses physical write/flush batching and may persist multiple successive certified units together. **Physical batching must not alter the deterministic operation history, signed roots, certification-unit boundaries or exact committed order.** Independently certified units retain their evidence and predecessor linkage. A longer-range certificate cannot be sliced to authorize an intermediate checkpoint. Batch thresholds need a bounded drain rule compatible with Marshal's delivery window, idle tails and shutdown; waiting for more Updates than the unacknowledged window allows would deadlock progress. Batch sizes and timing stay inside Storage.

Track the distinct milestones: ordered, executed, submitted, certified and durable. Only `Committed` advances the published durable frontier and permits covered ACKs or checkpoint cleanup. Receiving `Prepared`, accepting a certificate, or completing a write without the required durability barrier does not. A certificate may arrive before material: bound pending evidence or reject it for retry, while valid direct work continues.

Storage validates the full certificate subject, eligible epoch identities, distinct signatures, exact material and authoritative order; it rechecks the predecessor under writer authority before canonical mutation. Once mutation starts, task cancellation does not prove rollback. The retained operation must complete/reconcile or require recovery before another mutation. Recoverable state, outputs, applied identity, certificate evidence and provenance must agree. Recovery revalidates that linkage before emitting readiness; genesis uses the configured initial trust anchor.

### Event delivery and checkpoint installation

Events are a reliable retained handoff to the App owner, not lossy progress hints. The event path must be drained independently of submissions/signature work so a full queue cannot deadlock a waiting caller. Bound outstanding work and deliver backpressure; do not discard a `Prepared`, `Committed` or failure event. Duplicate/recovered events are idempotent by exact identity. A later `Committed` may cover earlier units only when the recoverable continuous prefix proves all of them; never drop a `Prepared` still needed for signing. Unexpected Storage task/channel closure stops new submissions and triggers recovery, not success.

On `Committed(applied)`, the App owner publishes the applied identity, reliably forwards `applied.checkpoint` to Executor and ACKs only covered retained Updates. Storage never ACKs Marshal or calls `Alignment::commit`; that method already records ordering promptly. Checkpoint handoff adds no execution wait or second installation ACK.

### Checkpoint handoff and execution skipping

`AppliedCheckpoint` contains the exact continuous certified ordered prefix, state/base/runtime identity, provenance and a usable pinned `StateView`. A height or root alone is insufficient. `DurableApplied` identifies the covered units and their recoverable evidence. Direct commit and certified import emit `StorageEvent::Committed`; startup recovery supplies the same established checkpoint through that retained handoff before dispatch. Assembly supplies the configured genesis checkpoint initially.

`execution.install_checkpoint(checkpoint)` is a concrete Executor operation, not another public trait. The App owner retains the newest compatible checkpoint until it is adopted; dispatch uses that latest retained base. Installation checks exact identity and read access, then replaces the memory base and execution generation. Skip covered queued work, retire covered/conflicting attempts, and ignore their late completions. Preserve compatible descendants and rebase their remaining overlays without applying covered changes twice. Repeated exact checkpoints are idempotent, older compatible ones never rewind execution, and conflicting ones are rejected.

Physical memory cleanup waits until live workers/readers/descendants no longer require an overlay or old view. Storage independently retains disk material needed by pinned views, queries, recovery and result serving. Logical retirement is not permission to free still-referenced data.

For example, if Executor is at A and Storage imports a certified durable ABC checkpoint, install it, skip local B/C and start D from ABC. If Executor already computed ABCD while Storage commits AB, adopt AB as the base, release covered A/B overlays when safe and preserve compatible C/D work. No reexecution is required just because persistence lagged behind execution. A certificate without usable material or a failed/pending apply produces no `Committed` checkpoint.


## Diagram notation

Read a method-call arrow as **caller → owner: operation(inputs)** and its return as the value or completion condition. An event arrow identifies publisher and consumer; it is independent of the earlier method return. Boxes separate responsibilities; they do not require separate actors, channels or public traits. Flowcharts show dependencies; sequence diagrams show ordering within the illustrated path. An arrow to a worker does not block unrelated App work.

| Label | Meaning | Implementation responsibility |
|---|---|---|
| **[C]** | Existing Commonware interface at the [native pin](../baton/README.md) | Implement the existing App callbacks; reuse supplied Engine, Marshal, network and QMDB operations |
| **[A]** | App-owned processing | The operation or helper name explains the flow; it does not prescribe a private method, exported Rust API or actor count |
| **[P]** | Proposed App trait contract | External `Alignment` and `Storage`, plus crate-internal `AlignmentPolicy`; shared by PreCut and Baton, not existing native interfaces |
| **[X]** | Missing native extension obligation | Design and implement the native integration before claiming protected-prefix behavior; the shown names are not settled signatures |
| **wire** | Peer message | Existing native messages retain their native codecs; App transaction/report/direction/result message names denote logical content, with concrete codecs/channels still open |

Solid versus dashed sequence arrows distinguish calls from returns/notifications, not durability. Only the stated completion contract establishes durability. Overview flowcharts may group methods and omit arguments; their named methods map to the same contracts as the detailed sequence diagrams. Receiver-local names such as `stage_block`, `engine.start` and `barrier.durable` refer to the documented Mailbox, Engine and Barrier methods. Generic arguments and some configuration fields are omitted for readability; use the Rust reference and source for compilable signatures. `Receiver<T>` is returned by the async Automaton callback; `reply.send(value)` later resolves it. Returning the receiver and sending the verdict are different events.

A label describing internal behavior or a data dependency is not an additional Rust method. `[A]` calls are documented concrete App helpers, with implementation signatures left private. A coalesced dispatch wakeup is an internal notification with no required named event type. `[X]` hooks are explicitly unresolved native obligations, not implementable SDK signatures. `wire:` App message names describe logical content; their codecs remain open. The [message catalog](networking.md#messages-used-in-diagrams) records those distinctions. Do not interpret all arrows as public trait methods.

The practical implementation split is:

- **App entry points:** transaction ingress, existing `Automaton::propose/verify`, and existing `Reporter<Activity = marshal::Update<Block>>`.
- **App implementation:** pool, the selected PreCut or Baton scheduler, exact-parent execution, result handling and durable state integration. Ordinary scheduling and canonical execution share this implementation.
- **Reuse:** native Engine, Marshal custody/relay/backfill/ordered delivery/cursor, Commonware transport and concrete QMDB primitives.
- **Native extension still required for Baton:** authenticated proposal-policy adoption, validation, prefix preservation/continuation and matching Marshal recovery. App scheduling alone does not provide it.

## Values carried between App calls

These are logical contracts, **not new Rust struct definitions or wire schemas**. Keep existing native values, concrete batch handles and exact identities where possible.

| Diagram value | Meaning |
|---|---|
| `Admission`, `RetainedBatch` | Pool classification or selected bounded transactions. Selection retains transactions; neither value means canonical inclusion. |
| `PayloadVerdict` | `Valid`, permanent `Invalid`, or `Pending` for unavailable dependencies. It describes payload validity, separately from custody and speculative eligibility. |
| `Candidate` | Exact producer block/context plus the validity, custody and authentication facts available to App. The owner rechecks current lifecycle/ancestry/eligibility before scheduling. |
| `Job`, `job_id` | Exact block, completed valid execution parent, canonical base, runtime and input prefix, bound to a claimed local attempt and reserved resources. The producer's lane parent is not the execution parent. |
| `StateChanges` | Executor's sole state-data payload to Storage, accompanied by `ExecutedRange` metadata: deterministic application changes produced on the assigned parent. All data required as durable application effects by the chosen result contract must be represented here; concrete encoding remains open. No root, certificate, storage batch or Executor checkpoint. |
| `ExecutedRange` | Exact bounded ordered unit, predecessor state/prefix, runtime and accepted direct-job identity; separately bound by the owner, never an unverified numeric frontier. |
| `StateView` | Retained read-only capability for the exact installed base; Storage pins or fences its backing state for all reads. |
| `StorageError`, `StorageFailure` | Distinguish rejected/backpressured submissions, unavailable reads, invalid evidence and fatal mutable-store/durability failure; include exact unit/instance scope for asynchronous failure. No error authorizes ACK. |
| `ExecutionOutcome` | Local job completion envelope containing `StateChanges` or failure/cancellation. App owns the job/context/provenance binding; the envelope is not another storage payload. |
| `CanonicalInput` | Exact continuous input retained from one or more Updates. A bounded certification unit may span blocks; its whole authorized range is applied together. App retains every original ACK token. No prior candidate or verify call is required. |
| `PreparedState`, `PreparedImport` | Selected commitments/material with full input/base/runtime/output identity, actual batch ancestry and direct/imported provenance. Preparation alone grants no writer authority. |
| `DurableApplied`, `AppliedCheckpoint` | Durable completion carries a usable read-only canonical checkpoint with exact prefix/runtime/state identity, matching `f+1` evidence and provenance. Executor installs it to skip covered work. A root or QMDB flush alone is insufficient. |
| `LeaderContext`, `FrozenReports`, `PreparedDirection` | Exact epoch/view/history/canonical parent/rule/window/frontier, original signed report snapshot, and a completed valid direction choice. No execution-completion or ordering-finality proof is implied. |
| `VerifiedCertificate` | Matching execution statements with `f+1` distinct eligible signatures, independently checked against irrevocable exact input/base/runtime. It is neither a native ordering QC nor proof of local durability. |

## Existing interfaces to connect

| Existing operation | Who supplies it | App work / result |
|---|---|---|
| `Automaton::propose(ctx)` | App implements the Commonware trait | Return a receiver; select transactions, construct `TransactionBlock::from_context`, stage the full block, then send its body digest. |
| `Automaton::verify(ctx, body_digest)` | App implements the Commonware trait | Return a receiver; obtain the exact block, check payload and durable custody, admit eligible work, then send the validity verdict without waiting for speculation. |
| `digest_reply.send(body_digest)` / `verdict_reply.send(bool)` | Existing one-shot sender paired with the callback receiver | Resolve the pending proposal/verification result; closure is handled under the callback lifecycle, not as a durable event. |
| `Reporter::report(Update)` | App implements a typed Commonware reporter | Synchronously retain the full Update and ACK; return `Feedback` after handoff. The canonical worker handles it later. |
| `Context::header(body_digest)` then `header.block_ref::<H>()` | Native block types | Derive the full header then exact Marshal lookup reference; neither value is the body digest alone. |
| `stage_block(block)` / `Custody::wait()` | Marshal mailbox/token | Accepted storage work / durable custody respectively; `put_block(block)` combines both. |
| `subscribe_block(reference)` / `fetch_block(reference)` | Marshal mailbox | Subscription yields a block after custody; explicit fetch can yield buffered bytes before durability. Fetch alone is insufficient for `verify(true)`. |
| `Relay::broadcast(header_digest, ())` | Existing Marshal Relay | Local propagation submission with `Feedback`; it does not prove peer receipt. App needs no duplicate block service. |
| `Update.acknowledgement.acknowledge()` | Existing Exact token, called by App owner | Complete this retained delivery only after matching durable App application; Marshal separately persists its cursor. |
| `network.register/start`, `buffered::Engine::new/start`, `resolver::Engine::new/start`, `marshal::open`, `Service::relay/start`, `Engine::open/start`, `Running::ready` | Existing assembly interfaces | Wire App handles, buffer/resolver and native planes in [startup order](../e2e/recovery.md). Custody must work before native recovery verifies blocks. |
| `DatabaseSet`, concrete batches and `Barrier` | Existing storage primitives | Implement the App's exact-parent access, selected roots, writer fencing and recovery linkage around the [QMDB call sequence](../execution/qmdb.md#current-qmdb-apis). |

Native `Reporter<Activity>` observations are optional App hints, distinct from required `Reporter<Update>` delivery. No App execution method is added to Automaton. Its async callback can hand work to a cloneable owner handle; `&mut self` does not serialize every worker or establish a database writer fence.

## Private pool and execution calls

These rows explain behavior and completion behind the diagrams. Helper names are illustrative, not methods an SDK consumer must implement. Choose private decomposition inside the implementation; do not add forwarding traits just to reproduce every row.

| Proposed call | Owner and return | Completion, waiting and failure contract |
|---|---|---|
| `pool.admit(tx, source)` | App pool → `Admission` | Validate/classify selected, unselected or rejected input. Duplicates follow pool policy. Admission persistence and selection policy remain open. |
| `pool.select(ctx, limits)` | App pool → `RetainedBatch` | Select bounded eligible transactions without removing them as committed. |
| `pool.reconcile(applied_outcomes)` | App pool maintenance | Reconcile from durable canonical outcomes; schedule after publication, without making pool maintenance another ACK approval. |
| `app.check_payload(ctx, full_block)` | App payload code → `PayloadVerdict` | Check exact header/body/digest and application validity. Temporary absence stays pending; state-dependent transaction outcomes are handled by execution. |
| `app.admit_candidate(candidate)` | App scheduler owner → admitted, deferred, already known or ineligible | Recheck origin/role, live lifecycle, exact identity, ancestry/authentication and applied state. Preserve `F`; sort only eligible pending work. Saturated speculation does not make a valid custody verdict false. |
| `app.dispatch_ready()` | App scheduler owner → claimed `Job` or no work | Choose an eligible pending input with a completed valid exact parent and resources. Bind/claim the attempt before launching; missing parent/capacity stays pending. |
| `execution.execute(job)` | App execution worker → `ExecutionOutcome` | Execute transaction semantics under valid parent access. It runs independently of the verify verdict. Failure/cancellation must not become a reusable checkpoint. |
| `app.finish_job(job_id, outcome)` | App owner → accepted completion or ignored stale result | Accept once for the current attempt/context/ancestry. Settle reserved resources only after no launch or safe termination; canceling a waiter alone does not stop worker/DB access. |
| `execution.install_checkpoint(checkpoint)` | App owner forwards Storage output → Executor | Install a usable certified durable base, skip covered input and retire conflicting attempts. Idempotent and monotone by exact context, not height alone. No persistent state write or extra ACK. |
| `execution.branch(job.parent)` | Executor → in-memory overlay/checkpoint | Resolve the exact parent from the installed read-only base plus completed in-memory effects. Storage reads remain guarded by Storage; no mutable DB handle is transferred. |

PreCut uses admission, pending sorting and dispatch directly. Baton uses the same shared contracts while managing its report/direction policy internally. Workers and a canonical consumer may run concurrently, subject to one owner accepting transitions and one canonical writer.

## Private canonical and state calls

| Proposed call | Owner and return | Completion, waiting and failure contract |
|---|---|---|
| `app.retain_update(update)` | App Update inbox → `Feedback` | Synchronous, non-dropping indexed-block/token retention and worker wakeup. Size for the active delivery window and coordinate old retained Updates across floor resets. `Backoff` is not a retry request. |
| `app.apply_next()` | App canonical worker | Process the next bounded continuous certification unit. Matching certified durable replay can ACK without duplicate effects; gaps/conflicts stop advancement. Drive reconciliation, preparation, certification, commit, publication and ACK. |
| `execution.reconcile(input, canonical_base)` | App execution code → `StateChanges` | Reuse exact validated changes, finish or repair execution on the required parent. App canonical intake separately chooses a ready Storage import. Never substitute pending scheduler order for Marshal order. |
| Storage preparation after `submit` | Storage internals → `StorageEvent::Prepared` | Materialize accepted StateChanges and prepare deterministic commitments for the exact unit; no larger speculative suffix or durability promise. |
| `app.publish_applied(applied)` | App owner | Advance exact applied identity once, retain/forward `applied.checkpoint` to Executor and schedule pool reconciliation. ACK covered Updates after certified durability; checkpoint handoff adds no approval round. |
| `state.recover(config)` | Storage assembly → retained `StorageEvent::Committed(recovered)` or recovery failure | Recover linked state/output/applied-position/certificate/provenance, establish the pinned view and emit `recovered: DurableApplied` to the App owner. The owner installs `recovered.checkpoint` before dispatch. Configured genesis uses its initial trust anchor. |

The owner submits exact ordered changes through `Storage::submit`; result processing supplies evidence through `Storage::certify`. It handles `Prepared` and `Committed` events independently, without awaiting a flush inside execution. The ACK tokens stay with the retained delivery owner throughout these calls. `Storage` does not acknowledge Marshal directly. After a crash between App persistence and Marshal cursor persistence, replay verifies exact identity and recoverable certification evidence, then acknowledges idempotently. No new signature round is needed for a recovered certified applied record. Missing certification keeps input pending. The bounded signing unit must permit progress inside the delivery window; see the [canonical sequence](../e2e/canonical.md) and [certification units](../execution/interfaces.md#certification-units-and-delivery-progress).

## Baton behavior and native extension

Baton implements the same Alignment contracts as PreCut. Its report authentication, fixed-window collection, bounded direction selection and pending-order changes remain implementation behavior. The [policy rules](../baton/direction.md) define the invariants; no private method catalog is part of the SDK contract. The following native extension obligations are separate from that internal decomposition.

| **[X]** obligation named in diagrams | Native implementation still required |
|---|---|
| `leader_context()` | Expose authenticated leader-window/proposal context; producer Automaton context is not this hook. |
| `offer_policy(prepared)` | Make an already prepared candidate available without making native cut await reports or planning. |
| `adopt_policy(actual_context, ready_candidate)` | Validate availability/adoption, bind to actual parent/frontier, freeze the authenticated policy and preserve its exact leading execution sequence. |
| `validate_policy(proposal, actual_context)` | Validators verify the same policy interpretation and preservation; native continuation and Marshal order/recovery must carry it consistently. |

These four names describe obligations, not an approved hook API. Proposal encoding, adoption point, continuation and recovery proofs remain [open native design](../consensus/decisions.md). Before adoption, no prepared valid choice uses the actual-parent valid base path; after adoption, dropping the protected prefix as fallback is forbidden.

## Private result, import and lifecycle calls

| Proposed call | Owner and return | Required contract |
|---|---|---|
| `results.sign(statement, direct_evidence)` | App execution/result code → attestation | Bind irrevocable exact input/base/runtime/result and verified direct execution. Imported work cannot become an own direct signature. |
| `results.accept(evidence, canonical_context)` | App result code → `VerifiedCertificate`, pending or invalid | Verify full subjects and `f+1` distinct eligible signers independently of native ordering QC membership. Incomplete/missing context stays pending. |
| `state.verify_material(certificate, material, base)` | App state code → `PreparedImport`, pending or invalid | Verify exact applicability and retain candidate material inside Storage, keyed by its full certified target, before returning PreparedImport. Later certify resolves that same retained material and rechecks the base under the shared writer. Neither return grants writer authority nor justifies stopping direct work. This concrete import helper is not a fourth method on the proposed public Storage trait. |
| `app.open(config)` | App assembly → callback handles | Create retained Update/event ingress and wire dependencies before starting Marshal output. Storage recovery supplies the base separately; speculative lifecycle remains recovering. |
| `app.start(mailbox.clone())` | App assembly → running callback work | Make custody lookup and Update handling usable before `Engine::open`; recovery verify must be answerable. |
| `app.activate(native_ready, recovered_base)` | App owner → live speculative lifecycle | Activate only after successful native readiness and consistent App recovery; failed startup does not dispatch speculation. |

Follow [results](../e2e/results.md), [state sync](../e2e/state-sync.md) and [recovery](../e2e/recovery.md) for the actual call ordering. These operations stay inside App; Baton planning does not gate them.

<a id="how-to-read-the-rust-traits"></a>

## Call flow and completion

| Entry | App work | Completion boundary |
|---|---|---|
| Client / transaction peer | Validate and retain pool candidates | Admission, not canonical inclusion |
| `Automaton::propose` | Select batch, construct producer block, stage in Marshal | Body digest through the returned receiver; local custody verification follows |
| `Automaton::verify` | Validate payload/custody and record usable scheduler input | Validity through the returned receiver; no speculative execution wait |
| Optional native `Reporter<Activity>` | Correlate native observations | Best-effort hint |
| Marshal `Reporter<Update>` | Retain canonical index, full block and ACK | Immediate local handoff |
| Marshal ordered-input handoff | Retain complete Update and reflect exact continuous ordering in `Alignment::commit` | Ordering recorded without awaiting execution, certification or durability |
| App canonical worker / Storage events | Submit ordered execution changes or verified import; Storage verifies `f+1` and batches durable application | `Committed` supplies a checkpoint to Executor and permits covered ACKs; Marshal cursor persistence is separate |
| Peer execution messages | Validate exact results and applicable state material | Result certification or safe import, independent of Baton reports |

Existing cloneable handles and Commonware channels connect the App owner to asynchronous work. The completion contracts above explain each handoff without fixing private function names or adding a new public framework.

## Read the detailed contract once

| Question | Canonical page |
|---|---|
| What goes inside App, and what does Marshal already implement? | [Application connection guide](../baton/README.md) |
| What does verify do; how are local, remote, recovery and repeated calls handled? | [Callback behavior and pseudocode](../baton/interfaces.md#automatonverify-validity-custody-and-scheduling) |
| What exactly waits for ACK and how does crash redelivery work? | [Update, queue and durability contract](../baton/interfaces.md#marshal-reportupdate-canonical-input-and-ack) |
| Which digest, parent, index or context does a value identify? | [Terminology](glossary.md), [Rust types and callbacks](rust-interfaces.md) |
| How do Pre-cut and Baton differ, and what native work remains? | [Alignment and policy gap](../baton/direction.md) |
| How are execution effects, roots, writer access and imported results connected? | [Executor](../execution/README.md), [Storage](../execution/storage.md), [result processing](../execution/interfaces.md), [QMDB](../execution/qmdb.md) and [state sync](../execution/state-sync.md) |

Use types from one resolved Commonware dependency graph; matching names or digest bytes across versions do not establish Rust type compatibility. Native Activity and Marshal Update need separate typed reporter handles when both enter one App, because Reporter has one associated Activity per implementation.

## Finding calls in E2E cases

| Case | Page |
|---|---|
| Transaction through durable application | [Normal flow](../e2e/normal.md) |
| Propose, relay, verify and missing body | [Body exchange](../e2e/block-body.md) |
| Producer DA, leader proposal and native finality | [Native consensus](../e2e/native-consensus.md) |
| Report windows, direction and proposal freeze | [Leader](../e2e/leader.md) |
| Pending sorting and exact-parent dispatch | [Scheduling](../e2e/reschedule.md) |
| Update, reuse/repair, durable apply and ACK | [Canonical flow](../e2e/canonical.md) |
| Startup, restart and redelivery | [Recovery](../e2e/recovery.md) |
| Result signatures and certified import | [Results](../e2e/results.md), [State sync](../e2e/state-sync.md) |
