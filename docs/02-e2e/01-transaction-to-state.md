# Transaction to durable state

<a id="normal-flow-transaction-to-applied-state"></a>

**One App owns the transaction pool, scheduler and transaction execution.** Multimmit calls the App through `Automaton`; Marshal stores and broadcasts complete blocks and delivers their finalized order through `Reporter<Update>`. App storage applies the matching execution result durably.

## Whole path at a glance

```mermaid
flowchart TB
    T["Transaction"] -->|"[A] TxPool::admit"| P["Retain by type + share"]
    N0["Multimmit"] -->|"[C] Automaton::propose(context)"| S["[A] TxPool::select<br/>epoch/lane → eligible types → Payload"]
    P -.->|"retained transactions"| S
    S --> B["Execution builds block<br/>[C] Marshal.stage_block"]
    B --> V["[C] Automaton::verify<br/>validity + durable custody"]
    V -->|"eligible candidate"| X["[P] Alignment::admit / AlignmentPolicy::next<br/>early execution"]
    V --> N["Native signing / DA / finality"]
    N -->|"[C] report(Update)"| O["Canonical execution<br/>reuse or repair exact order"]
    X -.->|"matching work"| O
    O -->|"[P] submit → Prepared"| C["Execution f+1 certification<br/>[P] Storage::commit"]
    C -->|"[P] Committed"| A["Install checkpoint + ACK"]
```

[Open full-size diagram](../assets/diagrams/diagram-34.svg) · [PNG](../assets/diagrams/diagram-34.png)

The early-execution branch is optional. Canonical delivery can be the App’s first encounter with a block and never depends on prior speculation.

Read after [component responsibilities and communication](../01-overview/01-architecture.md#responsibilities-and-communication). This case follows `propose/verify`, Alignment admission/selection, `report(Update)` and Storage requests/events through timing, completion and failure paths.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../03-reference/01-communication-contracts.md#diagram-notation) for contracts and implementation ownership.

## Full E2E: transaction input to canonical state

This successful path shows one possible completion order. Verification errors and pending data are expanded in [body exchange](02-block-exchange.md); replay, gaps and apply failures are expanded in [canonical application](06-ordered-application.md).

<details>
<summary>Detailed interface sequence</summary>

```mermaid
sequenceDiagram
    participant C as Client / tx peer
    participant A as Execution / coordination
    participant N as Multimmit Engine
    participant M as Multimmit Marshal
    participant P as TxPool / payload
    participant E as Execution / execution workers
    participant S as Storage
    C-->>A: wire: Transaction(tx)
    A->>P: [A] pool.admit(tx)
    P-->>A: Ok(Admission): duplicate = false
    A-->>C: Admission: valid transaction retained
    Note over A,P: Retain by type and share across lanes, detailed admission cases follow below
    N->>A: [C] Automaton::propose(ctx).await
    A-->>N: Receiver[BodyDigest]
    A->>P: [A] pool.select(ctx, limits)
    P->>P: Resolve eligible types for epoch/lane and pack ready transactions
    P-->>A: Payload
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
    S-->>E: [P] StorageEvent::Prepared(prepared), via App routing
    E->>E: [A] results.sign(statement, direct_evidence)
    Note over E: Exchange peer attestations for the same full subject
    E->>E: [A] results.accept(evidence, canonical_context) -> verified or pending

    Note over A,S: Missing certification keeps work pending before state application
    E->>S: [P] Storage::commit(verified)
    S-->>E: Ok(()), commit accepted, not durable
    S->>S: Match verified subject, prepared material and current base, apply durably
    S-->>A: [P] StorageEvent::Committed(applied)
    A->>A: [A] app.publish_applied(applied)
    A->>E: [A] execution.install_checkpoint(applied.checkpoint)
    A-->>M: [C] update.acknowledgement.acknowledge()
    Note over A,M: Pool maintenance is scheduled, no extra pool ACK
```

[Open full-size diagram](../assets/diagrams/diagram-02.svg)

</details>

Speculative execution may finish before or after `Update` arrives. `verify(true)` establishes payload validity and custody; it does not wait for that execution. For local blocks, candidate admission must respect the fact that this verify occurs before native header signing. The [callback contract](../../execution/README.md#coordination) covers peer, local and recovery requests.

For direct canonical work, the App worker awaits the selected unit's execution/reuse and then `Storage::submit` acceptance before advancing to the next ordered unit. The `Ok(())` reply in the diagram ends that synchronous handoff; later preparation, certification and disk flush remain independent. See [execution-to-Storage completion](../../execution/README.md#execution-execution-to-storage-completion).

Marshal already delivers the ordinary native total order. The App reconciles retained work against these Updates and its canonical worker applies their exact sequence. It does not sort Updates again or require an earlier speculative candidate. In Baton mode, reports and direction operate alongside this path; native proposal-policy integration remains [open](../03-reference/05-native-open-decisions.md).

`Alignment::commit(ordered)` records ordering before execution or certification completes. Execution then verifies a matching `f+1` result certificate before requesting Storage commit, for direct work and imports alike. Completed speculation may remove computation, but cannot bypass certification. Native cut formation continues independently; insufficient signatures retain pending work and withhold state application and ACK. Bounded multi-block signing follows the certification-unit contract without waiting for per-block application inside the unit.

## Tx lifecycle: new, duplicate, and invalid transactions

<details>
<summary>Detailed interface sequence</summary>

```mermaid
sequenceDiagram
    participant C as Client / tx peer
    participant A as Execution / Node ingress
    participant P as Local TxPool
    participant Q as Peer Node / TxPool
    participant N as Multimmit Engine
    participant W as Execution canonical owner
    C->>A: wire: Transaction(tx)
    A->>P: [A] TxPool::admit(tx)
    P->>P: Validate transaction and exact duplicate identity
    alt Invalid transaction
        P-->>A: Err(AdmissionError::Invalid)
        Note over A,P: No retained payload or peer relay
    else Already-retained valid transaction
        P-->>A: Ok(Admission): duplicate = true
        Note over A,P: No reinsertion or immediate relay loop
    else Valid new transaction, capacity unavailable
        P-->>A: Err(AdmissionError::Backpressure)
        Note over A,P: Caller keeps tx for retry, no successful retention claim
    else New valid transaction can be retained
        P->>P: Retain by transaction type regardless of local lane eligibility
        P-->>A: Ok(Admission): duplicate = false
        A-->>Q: wire: Transaction(tx), all retained types
        Q->>Q: [A] TxPool::admit(tx), validate and retain by type
        Note over P,Q: Type-to-lane assignment only affects proposal selection
    end
    A-->>C: Admission result under the ingress contract
    N->>A: [C] Automaton::propose(ctx)
    A-->>N: Receiver[BodyDigest]
    A->>P: [A] TxPool::select(ctx, limits)
    P->>P: Choose eligible type pools for epoch/lane, pack ready transactions
    P-->>A: Payload with stable bounded bytes
    A->>A: [C] TransactionBlock::from_context(ctx, body)
    Note over A,N: Staging and digest reply follow the full E2E path above
    Note over A,P: All types stay retained and shared across selection and epoch changes
    W->>W: [A] app.publish_applied(DurableApplied)
    W-->>P: [A] TxPool::reconcile(PoolUpdate)
    P->>P: Reconcile type pools from durable outcomes of any lane
    Note over W,P: Pool maintenance does not gate canonical ACK
```

[Open full-size diagram](../assets/diagrams/diagram-03.svg)

</details>

`admit` validates, deduplicates and retains transactions by type independently of lane eligibility. `Automaton::propose` triggers Execution to call `TxPool::select`, which resolves eligible type pools from the epoch/lane and returns a bounded Payload. All retained types remain shared; proposal eligibility is not a stored transaction class. Duplicate metadata, invalid input and retryable backpressure are separate admission outcomes. Type assignment and physical storage remain developer-owned choices. Duplicate suppression in one pool does not establish global exactly-once execution when multiple producers include the same transaction.

See [transaction admission and selection](../../tx-pool/README.md#pool) for the TxPool contract, the [backend survey](../03-reference/04-pool-backends.md) for reuse options, and [canonical application](06-ordered-application.md) for durable outcome handling. The node assembly is grounded in the current [log-multimmit example](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/examples/log-multimmit/src/node.rs#L242); its synthetic payload generator does not implement this transaction application.
