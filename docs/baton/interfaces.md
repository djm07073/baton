# App: native callbacks

<a id="application-callbacks-and-scheduler-behavior"></a>

<a id="interface-overview"></a>

**App callback handles receive payload requests from Multimmit and ordered input from Marshal.** They retain request/input identity and hand long work to bounded jobs while the shared App owner handles other events. The [Rust reference](../overview/rust-interfaces.md) contains existing signatures; private handler names below describe their behavior.

## Communication interfaces

| Direction | Interface | Completion means |
|---|---|---|
| Multimmit → App | `Automaton::propose(ctx)` | Returned receiver later resolves to a staged body's digest |
| Multimmit → App | `Automaton::verify(ctx, body_digest)` | Returned receiver later resolves validity/custody, independently of speculative execution |
| Marshal → App | `Reporter::report(Update)` | Synchronous retention of indexed input and its token; ACK follows certified durable application |

The sections below expand each handler's ownership, waits and failure paths.

The [interface map](../overview/interfaces.md) gives the shared call names, inputs/results and implementation owners used by the E2E Mermaid diagrams. The pseudocode here expands those contracts; it does not require an additional set of methods.

Cloned callback handles route to the same App state. For `propose` and `verify`, return the receiver promptly and run the pending request work shown below as bounded async jobs. Body/custody waits leave the App owner free to handle intake and completions; completed jobs re-enter that owner before updating shared state. The [existing example mailbox](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/examples/log-multimmit/src/application/mailbox.rs#L148) uses this handoff.

## Assemble before opening the engine

Restore App's canonical state/applied binding, or initialize the agreed App genesis; Marshal's index 0 is genesis and is never delivered as an Update. Use the [existing assembly](../reference/integration.md#body-assembly-to-implement), with App custody and retained Update intake available before `Engine::open` invokes recovery verification. Gate speculative dispatch during recovery, not the custody answers needed to finish open.

Pass App's Automaton, Marshal's relay and the native activity reporter into Engine. Start its planes, await native readiness and App's recovered base, then activate scheduling and re-evaluate retained candidates; another successful verify callback is not guaranteed. Native `ready` is only a startup milestone. The example supplies wiring, not the transaction executor or durable Update consumer.

Keep failed startup, sibling-service lifetimes and shutdown under the [existing node owner](../reference/integration.md#service-lifetimes). Checkpoint imports must also follow the [floor-to-App-state authority contract](../execution/state-sync.md#coordinate-the-app-checkpoint-with-marshal), including authentication before using `Start::Floor`.

## Automaton::propose: pool to staged block

```text
on propose(context, digest_reply):
    select a bounded batch from App's selected transaction candidates
    build body and TransactionBlock::from_context(context, body)
    await Marshal.stage_block(full_block) -> accepted custody handle
    resolve digest_reply with full_block.header().body_digest()
    // Native code next performs its local verify/custody check.
    // Do not retire transactions merely because they were selected or proposed.
```

`stage_block(...).await` accepts custody work; the returned custody handle is the later durable-completion boundary. `put_block(...).await` combines staging and waiting. The native local `verify` must confirm validity and durable custody before local signing. If App promises durability earlier in its own propose implementation it may wait there, but selecting a batch or accepting staging alone is not a disk flush.

Transient absence of transactions or backpressure follows the callback's request lifetime and configured producer behavior. Do not invent an empty-block policy or treat local storage failure as successful custody. A failed mutable-storage operation is a fatal/recovery condition for that storage instance.

## Automaton::verify: validity, custody and scheduling

```text
on verify(context, body_digest, verdict_reply):
    expected_header = context.header(body_digest)
    reference = expected_header.block_ref() using App's configured hasher
    locate/subscribe to the exact full block through Marshal using reference
    // Start explicit fetch where needed, without awaiting subscription first.
    await the required body; validate body digest, context and payload rules
    if permanently invalid:
        mark this candidate and dependent speculative eligibility unusable
        resolve verdict_reply(false)
        return
    if temporary data/dependency is missing:
        retain the live request and reply until data arrives or native cancels
        return to the event loop
    establish durable custody, unless the successful subscription already did so
    on the App owner, recheck current applied position, lifecycle and eligibility
    retain/deduplicate still-useful candidate metadata
    if currently live, not already applied, and eligible for speculation:
        update the selected scheduler's pending candidates
        arrange a later dispatch when exact parent/resources are ready
    resolve verdict_reply(true)
    // No wait for transaction execution, result roots, reports or directions.
```

The actual `verify` future returns `oneshot::Receiver<bool>`; App later sends the verdict through the paired sender. `false` means permanent invalidity, not “body has not arrived,” “worker busy” or “speculation failed.” Keep temporary uncertainty pending. A successful `subscribe_block` already establishes durable custody; do not introduce a redundant second flush solely for scheduling. By contrast, `get_block` or `fetch_block` can return buffered material before its storage sync completes, so successful lookup/fetch alone cannot justify `true`. See [the body API guide](../consensus/block-body.md#use-marshals-existing-body-apis) for the fetch-first custody path and superseded subscription handling.

The shared Automaton documentation describes single-shot verification and terminal closure. Current Multimmit's remote chain plane has a narrower implementation exception: a closed receiver becomes `Unavailable`, eligibility returns to `Ready`, and verification can be dispatched again within the same run. Local uncanceled custody failure is fatal; recovery also requires `true`. Do not rely on closure as a portable retry mechanism, and deduplicate repeated live calls as well as recovery. [Remote dispatch](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/consensus/src/multimmit/actors/voter/chain_plane.rs#L522), [eligibility transition](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/consensus/src/multimmit/machine/eligibility.rs#L352).

Use payload validity rules that can legitimately be checked at this boundary. Producer context does not specify the final cross-lane state. State-dependent transaction outcomes belong to execution against the chosen exact parent; a speculative revert or conflict does not retroactively make an otherwise valid payload malformed.

The scheduler's pending entry is not a finished execution result. Native remote dispatch can occur while the producer parent's own validation is pending; it does not prove all predecessor bodies are ready. App checks candidate ancestry/body readiness and the execution checkpoint separately. Admission is an inexpensive local state update. Keep speculative capacity separate from custody and canonical delivery: when a speculative budget is exhausted, retain/coalesce a deferred candidate where the chosen bounded policy permits, or forgo that speculative opportunity. Do not send `false` or wait for execution just to obtain a speculation slot. Marshal's later canonical Update remains authoritative even if a block was never speculated.

Deduplicate candidates across concurrent verification, local proposal, activity hints and recovery. Repeated verification refreshes candidate facts; it does not reset an existing execution attempt to pending. Match exact epoch/context/block identity and exclude already applied speculative work using current owner state after asynchronous custody completes. Canonical application may have advanced while verification waited. Already-applied or unscheduled does not mean invalid payload: complete a still-required custody verdict honestly while skipping speculative admission. If a required producer parent is permanently invalid, dependent candidates remain ineligible even if their own payload bytes passed validation. The callback has no origin parameter; qualify native role/lane and App lifecycle as described in [the entry guide](README.md#when-speculative-execution-begins). Missing optional activity hints may reduce speculation but cannot stall canonical delivery.

### Dispatch after verification

```text
when a usable candidate or worker completion changes App state:
    accept a completion only for the still-current execution attempt, once
    reject cancelled, retired, duplicate or context-mismatched completion
    preserve the valid completed/current local prefix F
    sort only eligible pending candidates using G
    if no valid execution parent checkpoint is ready:
        leave dependent work pending
    else if a worker slot is available:
        recheck eligibility and choose the next pending block
        bind the job to exact input state, runtime, ordered prefix and generation
        on App owner, claim that pending job and reserve its resources
        record the current attempt before launching it
        start transaction computation on that parent
```

Computation runs through Commonware runtime workers, outside the native callback and App's short admission handlers. App retains the execution tree, completed effects and worker fences; the scheduler chooses pending work using that state. Claiming the job prevents another wakeup from starting the same attempt; distinct exact-parent repair work remains distinguishable. A failed launch or a current attempt ending without reusable output must settle its ownership once and re-evaluate work from the last valid checkpoint. Release claims/resources only after work never started or has safely terminated; dropping a waiter alone does not prove submitted CPU work stopped. Retain required canonical input for the chosen repair/recovery path. Correlate the current local attempt as well as exact input/parent/runtime: a matching generation alone does not make duplicate completion safe. The scheduler does not interpret the native producer header's `parent` as an application state root.

## Marshal report(Update): canonical input and ACK

Marshal calls `Reporter::report` synchronously with an `Update` containing `index`, `Arc<TransactionBlock>` and `acknowledgement: Exact`. Acknowledge that exact delivery only after App has verified a matching `f+1` execution-result certificate and durably applied the covered input.

The trusted local Marshal stream supplies the ordering authority; App checks its exact range against the recovered application checkpoint. Update is not a portable finality proof and does not carry execution state, runtime, result certificate or Baton policy. A peer-sent object with the same fields cannot substitute for this local callback. External evidence export remains a separate open contract, not a second verifier required for ordinary App delivery.

```text
report(update):
    transfer the entire Update, including its Exact token, to App's retained inbox
    reflect new continuous finalized ordering through Alignment::commit
    return local Feedback immediately

App canonical worker:
    preserve the next continuous ordered unit and its exact ACK tokens
    if recovered certified durable identity covers it, ACK replay idempotently
    otherwise reuse/complete/repair its execution, or choose a verified peer import
    forward exact executed range and StateChanges through Storage::submit
    acceptance retains changes; continue execution without awaiting a flush
    retain input on rejection, missing certification or storage failure

on StorageEvent::Prepared:
    App result handling signs direct work, collects evidence and calls Storage::certify

on StorageEvent::Committed(applied):
    reconcile the exact certified durable prefix with retained Updates
    retain/forward applied.checkpoint so Executor skips covered input
    refresh execution state and schedule pool maintenance from durable outcomes
    acknowledge only the exact covered Updates
```

An Update may be App's first encounter with the block, including for an observer or skipped speculation. Canonical processing cannot require an earlier `verify` call or scheduler entry: it performs the application checks needed for execution, or reuses exact validated work. This does not add another Automaton callback.

The public [`Storage`](../overview/interfaces.md#proposed-storage-interface) contract owns certificate verification and durable application. Executor supplies StateChanges with exact executed-range metadata; Storage prepares storage commitments and App result handling collects evidence. The retained Update owner forwards the `Committed` event's checkpoint to Executor and handles ACK. Storage submission can precede certification and durability; neither acceptance alone authorizes ACK. A call into this trait never waits for Alignment approval.

Do not put application execution inside `report` or return an async future from it. App can pipeline receipt and computation while preserving canonical application order. Reports/direction planning cannot approve or gate this worker. App's capacity policy must also let required custody and canonical execution, root preparation and durability make progress under sustained optional load. Speculation, report verification and planning cannot indefinitely monopolize the resources that work needs.

| Event | What it proves | What can continue |
|---|---|---|
| `report(Update)` returns | App accepted the delivery into its retained path | Native consensus and other delivery slots |
| App finishes matching transaction work | A result exists for the exact execution context | Root preparation and result signing; canonical apply awaits certification |
| App verifies matching `f+1` certificate | Exact ordered input/base/result is certified | Writer may apply matching material after its authority checks |
| App completes certified durable apply and ACKs | App can recover this applied input and certification evidence | Marshal releases acknowledged window capacity in contiguous order |
| Marshal syncs its delivery cursor | Marshal can recover its acknowledged delivery position | Recovery from the persisted cursor and permitted pruning |

Within one active delivery window, Marshal can report up to `max_pending_acks` outstanding Updates. A full window pauses further App delivery; native voting, leader finality and producer progress do not wait for these ACKs. Retain every accepted Update and its Exact token in a queue sized for that window. `Closed` terminates delivery; `Backoff` does not retry a dropped Update. Every Exact clone must acknowledge, and dropping an unacknowledged clone cancels its live waiter.

ACK is an in-memory signal after App durability. Marshal subsequently persists its own cursor, so a crash between those boundaries can redeliver already-applied input. Compare recovered position and exact identity before ACKing replay; do not duplicate effects. A gap or conflicting identity stops application for recovery/error handling. An included block with a missing body is never an empty slot.

For floor/import transitions, the active-window bound does not cover old Updates still retained by App after Marshal resets its window. Update has no generation field. Serialize the transition through App ownership: fence/reconcile old inputs and tokens against matching durable App state, or explicitly budget bounded overlap before accepting a fresh window. The [detailed handoff](../execution/state-sync.md#coordinate-the-app-checkpoint-with-marshal) remains implementation work.

Result certification precedes canonical application for both direct and imported work: irrevocable exact input/base/runtime plus `f+1` distinct matching execution signatures. While signatures are missing, retain prepared work without canonical state apply or ACK. Signing must precede application, and the next certification unit must fit the active delivery capacity. Within a unit, private execution checkpoints allow progress before application. [Execution result contract](../execution/interfaces.md#result-certification-inside-executor).

<a id="finalized-state-progress-notifications"></a>

## Native activities and optional local notifications

Route native `Activity` directly to Marshal through its [existing mailbox reporter](../consensus/README.md#native-activities). App may additionally consume `TransactionProposed`, `ProtocolAccepted`, finality or history observations for candidate correlation and advisory scheduling. These are best-effort, idempotent hints; App must not build a second finality/delivery engine from them.

Shared execution progress is ordinary App state: after canonical apply, remove applied pending entries and rebase/fence conflicting speculative work without calling `Alignment::commit` again. Applied-progress coordination adds no extra ACK protocol or scheduler approval. Result signatures, certificates and state-sync material travel directly between peer application execution components; Baton report collection is outside that path.
