# Transaction to durable state

<a id="normal-flow-transaction-to-applied-state"></a>

**One App owns the transaction pool, scheduler and transaction execution.** Multimmit calls the App through `Automaton`; Marshal stores and broadcasts complete blocks and delivers their finalized order through `Reporter<Update>`. App storage applies the matching execution result durably.

Read after [component responsibilities and communication](../README.md#components-and-their-connections). This case follows `propose/verify`, Alignment admission/selection, `report(Update)` and Storage requests/events through timing, completion and failure paths.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../overview/interfaces.md#diagram-notation) for contracts and implementation ownership.

## Full E2E: transaction input to canonical state

This successful path shows one possible completion order. Verification errors and pending data are expanded in [body exchange](block-body.md); replay, gaps and apply failures are expanded in [canonical application](canonical.md).

```mermaid
sequenceDiagram
    participant C as Client / tx peer
    participant A as Alignment layer / App owner
    participant N as Multimmit Engine
    participant M as Multimmit Marshal
    participant P as App pool / payload
    participant E as App Executor
    participant S as Storage
    C-->>A: wire: Transaction(tx)
    A->>P: [A] pool.admit(tx, source)
    P-->>A: Admission
    A-->>C: Admission: selected / unselected / rejected
    N->>A: [C] Automaton::propose(ctx).await
    A-->>N: Receiver[BodyDigest]
    A->>P: [A] pool.select(ctx, limits)
    P-->>A: RetainedBatch
    A->>A: [C] TransactionBlock::from_context(ctx, body)
    A->>M: [C] stage_block(block).await
    M-->>A: Ok(Custody): accepted, may not be durable yet
    A-->>N: digest_reply.send(body_digest)
    N->>A: [C] Automaton::verify(ctx, body_digest).await
    A-->>N: Receiver[bool]
    A->>M: [C] subscribe_block(reference).await
    M-->>A: Ok(full_block): durable custody
    A->>P: [A] app.check_payload(ctx, full_block)
    P-->>A: PayloadVerdict::Valid
    A->>A: [A] app.admit_candidate(candidate)
    A-->>N: verdict_reply.send(true)
    N->>N: Native header signing
    N->>M: [C] Relay::broadcast(header_digest, ())
    opt Live candidate eligible, exact parent and capacity ready
        A->>A: [A] app.dispatch_ready() -> claimed Job
        A->>E: [A] execution.execute(job)
        E-->>A: ExecutionOutcome(job_id, StateChanges)
        A->>A: [A] app.finish_job(job_id, outcome)
    end
    N->>M: [C] Reporter::report(Activity)
    Note over N,M: Native finality and Marshal ordering run independently
    M->>A: [C] Reporter::report(Update)
    A->>A: [A] app.retain_update(update)
    A->>A: [P] Alignment::commit(ordered), ordering only
    A-->>M: Feedback: retained handoff
    A->>A: [A] app.apply_next()
    A->>E: [A] execution.reconcile(input, canonical_base)
    E-->>A: StateChanges: reuse / finish / repair
    A->>S: [P] Storage::submit(executed, changes)
    S-->>A: Ok(()), retained but not durable
    S-->>A: [P] StorageEvent::Prepared(prepared)
    A->>A: [A] App result path obtains certificate evidence

    Note over A,S: Missing certification keeps work pending before state application
    A->>S: [P] Storage::certify(certificate)
    S-->>A: Ok(()), admitted for verification, not durable
    S->>S: Verify f+1, subject, material and current base, apply durably
    S-->>A: [P] StorageEvent::Committed(applied)
    A->>A: [A] app.publish_applied(applied)
    A->>E: [A] execution.install_checkpoint(applied.checkpoint)
    A-->>M: [C] update.acknowledgement.acknowledge()
    Note over A,M: Pool maintenance is scheduled, no extra pool ACK
```

[Open full-size diagram](../assets/diagrams/diagram-02.svg)

Speculative execution may finish before or after `Update` arrives. `verify(true)` establishes payload validity and custody; it does not wait for that execution. For local blocks, candidate admission must respect the fact that this verify occurs before native header signing. The [callback contract](../baton/interfaces.md) covers peer, local and recovery requests.

Marshal already delivers the ordinary native total order. The App reconciles retained work against these Updates and its canonical worker applies their exact sequence. It does not sort Updates again or require an earlier speculative candidate. In Baton mode, reports and direction operate alongside this path; native proposal-policy integration remains [open](../consensus/decisions.md).

`Alignment::commit(ordered)` records ordering before execution or certification completes. Storage then requires a matching `f+1` result certificate before canonical state application, for direct work and imports alike. Completed speculation may remove computation, but cannot bypass certification. Native cut formation continues independently; insufficient signatures retain pending work and withhold state application and ACK. Bounded multi-block signing follows the certification-unit contract without waiting for per-block application inside the unit.

## Tx lifecycle: new, duplicate, and invalid transactions

```mermaid
sequenceDiagram
    participant C as Client / tx peer
    participant A as App ingress
    participant P as App pool / payload
    participant W as App canonical owner
    C->>A: wire: Transaction(tx)
    A->>P: [A] pool.admit(tx, source)
    P->>P: Bounded encoding, signature, static policy and duplicate checks
    alt Invalid transaction
        P-->>A: Admission::Rejected(reason)
    else Valid selected candidate
        P-->>A: Admission::Selected(tx_id)
    else Valid unselected candidate
        P-->>A: Admission::Unselected(tx_id)
    end
    A-->>C: Admission result under chosen ingress contract
    A->>P: [A] pool.select(ctx, limits)
    P-->>A: RetainedBatch(bytes, transaction_ids)
    Note over A,P: Selection does not retire transactions, cancellation keeps body bytes stable
    W->>W: [A] app.publish_applied(DurableApplied)
    W-->>P: [A] pool.reconcile(applied_outcomes)
    P->>P: Maintain chosen backend, replay missed maintenance before relying on readiness
    Note over W,P: Pool processing does not gate canonical ACK
```

[Open full-size diagram](../assets/diagrams/diagram-03.svg)

`admit` and `select` name App-internal pool operations, not a new public consensus trait. Selected/unselected are logical classes; their physical representation and policy remain open. RPC and peer input use the same validation and admission policy. Duplicate suppression in one pool does not establish global exactly-once execution when multiple producers include the same transaction.

See [transaction admission and selection](../tx/interfaces.md) for the App pool contract, the [backend survey](../tx/README.md) for reuse options, and [canonical application](canonical.md) for durable outcome handling. The node assembly is grounded in the current [log-multimmit example](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/examples/log-multimmit/src/node.rs#L242); its synthetic payload generator does not implement this transaction application.
