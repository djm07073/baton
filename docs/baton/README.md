# App: coordination and ownership

<a id="connect-the-application-to-multimmit"></a>

**App coordination connects native callbacks to the pool, Alignment, Executor, result processing and Storage.** It owns lifecycle, shared dispatch and retained Update tokens. Existing Multimmit Marshal supplies block custody, body exchange and ordered delivery.

The name `App` below means the application being designed. Inside it, **Alignment layer** owns callback/delivery handling and scheduling; **App Executor** owns transaction computation and in-memory checkpoints; **Storage** owns persistent state, certification checks and checkpoint import/recovery. The pool remains a separate App responsibility. These are responsibility groups, not mandated processes. The native `AppExecutor` runs local `Automaton::propose/verify` jobs; it does not execute application transactions. See the [responsibility map](../overview/interfaces.md#consensus-alignment-and-app-executor).

This design uses the current local Commonware checkout, [6233438](https://github.com/0xEyrie/monorepo/tree/6233438985d8249d2b2bc1204191d5d405652288). Earlier research against `534af0e` predates its Multimmit Marshal. Application scheduling, the transaction runtime and Baton policy integration described here remain to be implemented.

<a id="roles-and-responsibilities"></a>

## Responsibilities

| Owned responsibility | What the App owner coordinates |
|---|---|
| Native callback intake | Route payload construction/validation to the pool and custody path |
| Speculative dispatch | Admit eligible candidates, request a policy intent, recheck context and claim resources before launch |
| Ordered input | Retain complete indexed Updates and tokens; immediately reflect continuous ordering in Alignment |
| Independent completions | Accept current execution results and route Storage events without blocking other intake |
| Durable publication | Retain/forward usable checkpoints, ACK covered Updates and schedule pool maintenance |
| Lifecycle | Recovery readiness, stale-worker fencing, retained queues and shutdown |

The callback handles and shared owner are coordination roles inside App. The proposed `Alignment`/internal `AlignmentPolicy` and `Storage` contracts connect the policy and state responsibilities; concrete pool, execution and result helpers remain private. [Architecture](../overview/architecture.md) shows the composition.

Pre-cut and Baton share the same execution, certification and storage path. Pre-cut has no Baton instance, reports, directions or native policy change. Baton adds intended-order collection and advisory direction; authenticated native policy adoption remains separate unfinished work.

<a id="report-connection"></a>

<a id="what-app-implements"></a>

## Communication interfaces

| Direction | Interface or value | Completion means |
|---|---|---|
| Multimmit → App | `Automaton::propose/verify` | Receivers later yield a body digest or payload/custody verdict |
| Marshal → App | `Reporter::report(Update)` | Full indexed block and ACK token retained; `Feedback` completes the handoff |
| App → TxPool | `pool.select` / `pool.reconcile` | Stable proposal bytes / maintenance from durable outcomes |
| App → Alignment | `admit(candidate)`, `commit(ordered)` | Eligible candidate retained or finalized ordering reflected |
| App dispatch → Alignment policy | Internal `AlignmentPolicy::next(view)` | `JobIntent` or `None`; the owner still claims the job |
| App → Executor | `execution.execute/reconcile` | Exact-parent `ExecutionOutcome` / ordered StateChanges |
| App → Storage | `Storage::submit/certify` | Work or evidence retained; events complete later stages |
| Storage → App → Executor | `StorageEvent::Committed(applied)`, then `execution.install_checkpoint` | Certified durable identity published and a usable execution base handed over |
| App → Marshal | `update.acknowledgement.acknowledge()` | Matching delivery covered by certified durable App state |

[Native callbacks](interfaces.md) expand propose, verify and Update handling. The [communication reference](../overview/interfaces.md) defines exact values and completion/failure contracts. **[C]** is existing Commonware, **[P]** a proposed contract, **[A]** private App behavior and **[X]** unfinished native work. These names identify responsibilities rather than requiring one public trait per connection.

Use Marshal's existing `Relay` implementation. Its broadcast key is the **full header/block digest**, which locates the staged full block; this differs from the body digest returned by `propose` and passed to `verify`. The native wire carries signed headers separately from the Marshal body broadcast. A receiver may therefore have the header before the body. [Body flow](../consensus/block-body.md) explains these identities and custody stages.

`Reporter` has an associated `Activity` type. One concrete Rust type cannot implement it twice with two different associated types. Give the same App owner two small typed handles if it needs both native observations and Marshal deliveries. This is ordinary mailbox wiring, not another service layer.

## When speculative execution begins

The useful trigger is **a valid, usable block becoming known to App**, often during successful `verify`. App records that candidate, and the scheduler runs it when the required execution parent and resources are ready. The callback does not wait for the run.

`verify` is not a network-receive notification. Current Multimmit invokes it in three places:

- **Remote producer path:** native authentication/eligibility admits a header before the voter asks App to verify its payload. Duplicates, malformed packets and not-yet-eligible headers do not map one-to-one to calls.
- **Local producer path:** after `propose`, native code calls `verify` as a custody check **before signing the local header**.
- **Recovery:** `Engine::open` rechecks retained payload obligations before actors start.

Interpret successful **live** verification according to the node's role:

| Role | Candidate interpretation |
|---|---|
| Validator with a local producer lane | `context.chain() != local producer chain` can identify other-lane work |
| Nonproducing validator | Live validation still exists; there is no local lane to exclude |
| `Role::Observer` | No live per-chain validation; normally enter through Marshal Update unless an explicit authenticated early-candidate intake is added |

`verify` still does not mean a new network receipt. Before speculative admission, check App's current lifecycle and deduplicate exact block identity, excluding already-applied work. Track startup explicitly: the callback has no origin or recovery flag. These checks filter speculation, not the native custody verdict.

A local pre-sign body can be prepared locally. If report eligibility requires an authenticated native header, promote it only when that evidence is available. `TransactionProposed` and `ProtocolAccepted` are useful best-effort observations; missed hints may reduce speculation but cannot block canonical processing.

The scheduler separates **candidate admission** from **dispatch**. A body can be valid while its execution predecessor is unfinished. Admission updates pending work; dispatch later checks the exact parent checkpoint. Producer-header ancestry is lane ancestry, not the merged application's execution-state parent. [Callback behavior](interfaces.md#automatonverify-validity-custody-and-scheduling) makes this sequence explicit.

<a id="confirmed-order-delivery"></a>

## When canonical application begins

Marshal reports continuous indexed blocks in its existing native order. App treats each `Update` as authoritative ordered input, reconciles the speculative path, and uses an exact matching completed result when available. Otherwise it finishes or repairs the required work. It then prepares/applies the selected state through the application's canonical writer.

`Alignment::commit(ordered)` reflects the new finalized ordering as soon as the indexed input is retained. It waits for neither execution nor certification. Storage requires a matching verified `f+1` certificate before canonical application, then makes state, outputs, certificate evidence and applied identity recoverable. Only that certified durable completion releases the ACK token. Completed speculation can avoid rerunning transactions, but still needs certification. An ordering notification or queued block cannot authorize ACK.

Marshal can deliver several blocks before earlier ACKs arrive, up to `max_pending_acks`. A full window holds further application delivery; the native consensus engine does not wait for these ACKs to vote or finalize. Marshal persists its own delivery cursor separately after acknowledged progress. App must handle redelivery after a crash between application durability and Marshal cursor durability. See [the complete Update/ACK flow](interfaces.md#marshal-reportupdate-canonical-input-and-ack).

<a id="commonware-primitives-in-the-baton-layer"></a>

## Baton-specific work stays inside the application

Baton scheduling owns the report window, intended-order snapshot, bounded candidate evaluation and advisory direction. Reports bind epoch, view, history, canonical parent, rule, window and frontier. They express intention, not execution completion. Reuse the existing Clock, runtime tasks, crypto, codec and authenticated P2P for this work. Bound both retained messages and submitted CPU work; an unordered future pool alone is not a capacity limit.

The ordinary local rule is `F ++ sort_G(S_pending)`: preserve completed/current execution order and sort only eligible work not yet started. Cut readiness never waits for reports, a timer, planning, speculative execution or a direction reply. Details and the remaining protected-prefix integration obligation are in [Alignment behavior and direction](direction.md).

**An app-only Baton scheduler can guide speculation today; it cannot make its preferred order canonical by rearranging Marshal Updates.** The full research objective still needs an authenticated native policy/adoption mechanism and consistent history interpretation. Keep this explicit rather than adding a facade that makes the missing protocol work appear implemented.
