# Pre-cut baseline and comparisons

<a id="pre-cut-execution-without-baton"></a>

**PreCut is an app-owned scheduler that executes eligible blocks before their canonical order arrives from Multimmit Marshal.** The app then reuses matching work or repairs the changed suffix. It shares the transaction pool, body format/custody, execution backend, storage, result protocol and measured endpoints with the Baton app configuration.

PreCut has no Baton instance, report window, direction exchange or report-dependent native policy. It is a separate scheduling mode, not Baton with reporting disabled. Native Multimmit and Marshal supply the same ordinary canonical block order as Original. These are implementation-design documents; no runnable scheduler, E2E result or benchmark is claimed.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../overview/interfaces.md#diagram-notation) for contracts and implementation ownership.

## Architecture and responsibilities

```mermaid
flowchart TB
    N["Multimmit Engine"] -->|"[C] Automaton::propose / verify"| I
    N -->|"[C] Reporter::report(Activity)"| M["Multimmit Marshal"]
    subgraph A["PreCut inside App"]
        I["Alignment layer callback handles"] -->|"[A] pool.select / pool.reconcile"| T["App pool"]
        I <-->|"[P] Alignment::admit / commit<br/>AlignmentPolicy::next (internal)"| Q["PreCutAlignment"]
        I <-->|"[A] execution.execute / reconcile<br/>ExecutionOutcome / StateChanges"| E["App Executor: workers and checkpoints"]
        I <-->|"[P] Storage::submit / certify<br/>StorageEvent::Prepared / Committed"| S["Storage<br/>reads, f+1 and persistence"]
        I -->|"[A] execution.install_checkpoint"| E
        E <-->|"[P] Storage::read(view, key)<br/>cold state at pinned base"| S
    end
    I <-->|"[C] custody / ACK to Marshal<br/>report(Update) to App"| M
    I <--->|"wire: execution evidence / state material"| P["Peer Apps"]
    classDef reuse fill:#dbeafe,stroke:#2563eb,color:#172554;
    classDef fresh fill:#dcfce7,stroke:#16a34a,color:#14532d;
    class N,M reuse;
    class T,I,Q,E,S,P fresh;
```

[Open full-size diagram](../assets/diagrams/diagram-19.svg)

| Owner | Responsibility |
|---|---|
| App transaction pool | Admit/classify/retain txs, select bounded stable bytes in propose, maintain canonical outcomes |
| Native Multimmit | Producer headers, DA, proposals, votes, finality, tip extraction, extension and native recovery |
| Existing Multimmit Marshal | Durable body custody, relay, native proof/history verification, backfill, ordinary dense order and recoverable delivery cursor |
| App PreCut scheduler | Candidate deduplication, pending-only global ordering and dispatch with exact execution parents |
| App Executor | In-memory computation and checkpoints, branch reuse/repair; StateChanges are its sole state-data payload, paired with exact ordered-range metadata |
| Storage | Selected roots, writer/fencing, certificate verification, durable apply, state import/recovery and usable checkpoint handoff |
| App result processing | Sign accepted direct results and exchange execution evidence/material with peers |

`PreCutAlignment` implements public `Alignment` (`admit`, `commit`) and crate-internal `AlignmentPolicy` (`next`). The internal dispatch loop shares the latter contract with Baton. Both modes use public `Storage::read/submit/certify` and its retained events. Executor reads cold values at a pinned base and submits exact ordered changes without awaiting durability. `Committed` supplies a checkpoint to skip covered input and retire memory safely; import and recovery use the same handoff. Existing Marshal already implements the ordinary ordering/custody/delivery machinery that earlier drafts assigned to a new baseline attachment. The app implements transaction semantics and scheduling through concrete internal code.

## Inputs and Rust connection points

| Existing entry | PreCut app behavior |
|---|---|
| `Automaton::propose(context)` | Select from app pool, build/stage the complete block, return body digest through the one-shot response |
| `Automaton::verify(context, digest)` | Validate body/context and durable custody, then admit eligible speculative work without awaiting execution |
| Optional native `Reporter<Activity>` | Consume useful authenticated header/context observations; do not use lossy hints as the canonical stream |
| Marshal `Reporter<Update>` | Retain canonical index/block/ACK, reconcile/apply work asynchronously, ACK matching durable application |

Live verify-based intake applies to validators, including nonproducing validators. `Role::Observer` has no live per-chain validation planes; it executes canonical Updates unless an explicit authenticated early-candidate intake is added. `verify` also runs for local pre-sign custody and startup recovery. A callback is not automatically another-lane packet arrival. Qualify lifecycle, lane, exact authenticated context/reference, duplicate state and applied position before admitting a candidate. Header and body arrival order can differ. Preserve native authentication and body validation; no DA-certificate barrier is added merely for speculation. [Callback contract](../baton/interfaces.md).

```text
app handles successful verification:
    establish valid body and durable custody
    on App owner, refresh exact candidate facts
    if live, eligible and not already applied or claimed:
        admit candidate to PreCut pending work once
    reply true without awaiting speculative execution

PreCut dispatch:
    preserve completed/current path F
    sort only eligible unstarted pending work by G
    with valid exact predecessor and available worker capacity:
        on App owner, claim pending work and record its current attempt before launch
        launch transaction computation from that predecessor

app handles Marshal Update:
    retain Update and return promptly from report
    notify Alignment::commit of new continuous finalized order without waiting
    consume index in canonical order
    if this exact index/block has recovered certified durable application:
        ACK redelivery and return without applying again
    gather the next bounded certification unit within the delivery window
    reuse matching work or complete/repair using its private exact-parent checkpoints
    prepare the result and verify f+1 distinct matching result signatures
    if certification is missing or mismatched: keep work pending without state apply or ACK
    recheck certificate, material, predecessor and writer authority before mutation
    apply the authorized unit with recoverable identity, outputs and certificate evidence
    on App owner, publish the matching durable canonical advance once
    ACK after durable application
```

These are app-handler steps, not new public methods. The actual Rust signatures and typed callback-handle composition are in [Rust interfaces](../overview/rust-interfaces.md). Temporary body/dependency loss keeps verify pending; speculative capacity cannot make valid content permanently invalid. An unavailable speculative opportunity does not remove later canonical work from the Update path.

## Candidate arrival and speculative execution

Preserve completed/current order `F` on one valid path and use `F ++ sort_G(S_pending)` for local intent. PreCut produces no report, but uses the same eligibility and global pending rule as Baton's ordinary scheduling. With G = A→B→C, C then B admitted while A runs and before C starts yields ABC. Once C has started on A, a late eligible B yields ACB. Do not wait for hypothetical B or reexecute C solely because B arrived.

A producer-header parent is not an execution predecessor across lanes. Dispatch resolves the exact base, runtime and preceding input checkpoint. While that predecessor runs, admission only changes pending order. Incomplete, canceled or stale work cannot serve as a reusable completed result. Check [execution ancestry and QMDB applicability](../execution/qmdb.md#manage-the-branch-tree-for-execution-requests).

## Native ordering and repair on each confirmed update

**Repair is triggered by new canonical input from Marshal.** The ordinary native order is already implemented in the current Marshal; PreCut does not reconstruct it with a second app proof engine. Included missing blocks/history delay delivery/backfill; local absence is never evidence of an empty slot. Marshal presents complete blocks in canonical OutputIndex order.

| Local work / canonical prefix | App action |
|---|---|
| Completed ABC / canonical AB | Apply exact AB; retain compatible C when actual ancestry permits |
| Completed AC / canonical ABC | Reuse matching A; execute B on A, then repair C on AB or explicitly validate retained effects/outputs for AB and prepare C's checkpoint there |
| Completed ABC / canonical AXBC | Reuse matching A; execute X and repair affected B/C |
| No usable speculative result | Execute the exact ordered input from its valid predecessor |
| Duplicate Update after crash | Check exact durable applied identity and ACK without duplicate effects |
| Gap before next native output | Marshal backfills/holds delivery; native consensus continues |

A callback returning from `report` means retained intake and ordering notification, not durable apply. `Alignment::commit` reflects ordering immediately. Storage requires matching `f+1` certification before canonical application for both direct and imported work. Completed speculation cannot bypass that gate. The App owns each ACK until certified durability covers its input. A certification unit must fit the active delivery window; private computation within the unit precedes its application. Marshal delivery can fill while ACKs are outstanding, independently of native voting/finality. Do not ACK uncertified work to free capacity.

Canonical application and imports share one writer. Recheck descendants after the applied state base moves; stale completions cannot promote another branch. Persist recoverable state, outputs, applied index/block, certificate evidence and direct/imported provenance before ACK. Marshal later persists its own cursor, so recovery must handle redelivery. Pool maintenance stays internal and can reconcile from durable outcomes.

Repair in response to an unfinalized cut proposal is still an open additional policy. This page adopts confirmed Update processing, not automatic tentative-proposal repair. Global comparator/tie-break and resource budgets remain open; the existing Marshal base order must not be confused with that speculative global rule.

## Result certification and state sync

Use the same [result verification](../execution/interfaces.md#result-certification-inside-executor) and [certified state-sync path](../execution/state-sync.md) as Baton. State finalization requires irrevocable exact input/base plus `f+1` distinct eligible signatures on the same runtime/result statement. It precedes canonical application and durable/read readiness. Apply this gate in Original, PreCut and Baton comparisons alike.

App peers exchange results/material directly. A valid applicable certificate and material can replace unfinished local repair through the shared writer/fencing contract. Imported work cannot be signed as own direct execution. Keep this capability/configuration identical across comparisons and measure direct repair versus imported completion separately.

## Benchmark comparisons and measurements

| Configuration | Execution starts | Ordering coordination | Work after canonical input |
|---|---|---|---|
| Original | On ordered Update | Ordinary native Multimmit/Marshal | Execute and apply |
| PreCut | On eligible candidate plus valid execution parent | Same ordinary native order | Reuse/finish/repair and apply |
| Baton | Same ordinary speculative start | Reports/direction plus the still-required authenticated native policy extension | Reuse/finish/repair and apply |

Keep committee, topology, workload, pool/body behavior, native profile/cut cadence, global pending rule, execution/storage backend, root/checkpoint boundaries, worker limits and budgets matched. A Baton advisory-only run does not validate protected-prefix integration. Early proposals and frequent cuts are separate sensitivity experiments.

Measure latency distributions and throughput for ingress→ordering, ordering→verified result, primary state finalization and local durable/read readiness separately. Include computation, reused work, reexecution, canceled/discarded work, imports, hashing, queue/body/history waits and network/storage costs; distinguish required repair from transaction failure. Compare roots/outputs with deterministic execution of the same exact input/base. Lower post-cut work alone does not establish lower total cost.

## Implementation TODO and planned checks

- [ ] Assemble the app pool and real body with existing Automaton, Marshal mailbox/relay and ordered Update callback.
- [ ] Implement the independent PreCut scheduler and shared exact-parent execution/storage code.
- [ ] Connect durable Update application, idempotent crash redelivery and internal pool maintenance.
- [ ] Verify before/after-start A/C/B admission, local pre-sign and startup verify, missing body, duplicate candidate and saturated speculative capacity.
- [ ] Verify partial-prefix application, changed parent, stale completion, canonical writer/import races and direct/imported signature provenance.
- [ ] Verify native extension/view recovery and Marshal body/history gap handling without a second ordered-delivery implementation.
- [ ] Run matched Original/PreCut/Baton endpoint and cost measurements. Runtime checks and benchmarks are **Not run**.

The [shared execution decisions](../execution/README.md#open-decisions) remain open. Designing those choices and Baton's native protected-prefix integration are separate from reusing the already existing ordinary Marshal stream.
