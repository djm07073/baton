# Block publication and retrieval

<a id="block-body-send-fetch-and-receive"></a>

**Use the existing Multimmit Marshal for complete-block custody and exchange.** The App defines the transaction body codec, digest and validity checks. Native Multimmit handles signed producer headers, DA and ordering; the body travels through Marshal's buffered broadcast and backfill services.

Read after [Marshal custody](../02-components/02-marshal-custody.md) and [App callbacks](../02-components/05-app-callbacks.md). This case follows `stage_block`, `Relay::broadcast`, `subscribe_block/fetch_block` and the verification reply through timing, completion and failure paths.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../04-reference/01-communication-contracts.md#diagram-notation) for contracts and implementation ownership.

## Block lifecycle: mempool → propose → body dissemination

This is the successful local proposal path; the next diagram expands validation, pending data and custody failure.

```mermaid
sequenceDiagram
    participant N as Native producer
    participant A as Alignment layer / Automaton
    participant P as App pool / payload
    participant M as Multimmit Marshal
    participant B as Existing buffered transport
    participant R as Peer node
    N->>A: [C] Automaton::propose(ctx).await
    A-->>N: Receiver[BodyDigest]
    A->>P: [A] pool.select(ctx, limits)
    P-->>A: RetainedBatch
    A->>A: [C] TransactionBlock::from_context(ctx, body)
    A->>M: [C] stage_block(block).await
    M-->>A: Ok(Custody)
    A-->>N: digest_reply.send(body_digest)
    N->>A: [C] Automaton::verify(ctx, body_digest).await
    A-->>N: Receiver[bool]
    A->>M: [C] subscribe_block(reference).await
    M-->>A: Ok(full_block): durable custody
    Note over A,M: A retained Custody.wait() or put_block() can establish the same fence
    A->>P: [A] app.check_payload(ctx, full_block)
    P-->>A: PayloadVerdict::Valid
    A->>A: [A] app.admit_candidate(candidate)
    A-->>N: verdict_reply.send(true)
    N->>N: Native signing after successful local custody
    N->>M: [C] Relay::broadcast(header_digest, ())
    M-->>N: Feedback: local broadcast submission, not peer receipt
    M->>B: Existing staged-block broadcast
    B-->>R: wire: TransactionBlock(header, body)
    N-->>R: wire: SignedTransactionBlock(header, attestation)
    Note over N,R: Independent channels, arrival order is not guaranteed
```

[Open full-size diagram](../assets/diagrams/diagram-04.svg)

`stage_block().await` returns a `Custody` token after admission of storage work. Awaiting `Custody::wait()` establishes durable recovery; `put_block()` combines staging and that wait. Dropping the token does not cancel accepted storage work. The local native custody verify provides the required gate before header signing. An App may wait earlier in propose, but need not add a second durability path. [Marshal mailbox](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/consensus/src/multimmit/marshal/mailbox.rs#L327)

The body digest commits to the payload. The header digest identifies the native producer block, including its producer Context and body digest. `propose` returns the body digest; native `Relay::broadcast` requests propagation using the header digest. The existing Marshal Relay resolves that digest to the staged complete block. Its local Feedback does not prove remote receipt or custody, and even `Ok` can mean no matching staged block was found. [Relay completion](../02-components/12-networking.md)

## Body lookup, verification, and missing-content handling

```mermaid
sequenceDiagram
    participant N as Multimmit Engine
    participant A as Alignment layer / Automaton
    participant M as Multimmit Marshal
    participant E as App Executor / workers
    N->>A: [C] Automaton::verify(ctx, body_digest).await
    A-->>N: Receiver[bool]
    A->>A: [C] ctx.header(body_digest) -> header
    A->>A: [C] header.block_ref() -> exact BlockRef
    A->>M: [C] subscribe_block(reference).await
    opt Explicit retrieval needed, concurrent with subscription
        A->>M: [C] fetch_block(reference).await
        Note over A,M: Fetch may return buffered bytes, it does not prove durable custody
    end
    alt Required data or dependency still missing
        Note over N,A: Keep verdict pending until a verdict or native cancellation
    else Exact block available
        M-->>A: full_block (subscription or fetch result)
        A->>A: [A] app.check_payload(ctx, full_block) -> PayloadVerdict
        alt PayloadVerdict::Invalid
            A-->>N: verdict_reply.send(false)
        else PayloadVerdict::Pending
            Note over N,A: Retain live request, do not return false
        else PayloadVerdict::Valid
            opt Fetch won and durable custody not yet established
                A->>M: [C] put_block(full_block).await
                M-->>A: Ok(()) or custody error
                Note over A,M: On success retire superseded subscription, no redundant put after successful subscription
            end
            alt Custody failed
                Note over N,A: No true verdict, stop failed local storage path and recover
            else Durable custody established
                A->>A: [A] app.admit_candidate(candidate)
                A-->>N: verdict_reply.send(true)
                opt Eligible live Job with ready parent and resources
                    A->>A: [A] app.dispatch_ready() -> claimed Job
                    A->>E: [A] execution.execute(job)
                    E-->>A: ExecutionOutcome(job_id, StateChanges)
                    A->>A: [A] app.finish_job(job_id, outcome)
                end
            end
        end
    end
    Note over N,A: Remote, local pre-sign and recovery calls share this interface
    Note over A,E: Custody errors never become true, execution completion does not gate verify
```

[Open full-size diagram](../assets/diagrams/diagram-05.svg)

`subscribe_block` waits for local storage or buffered ingress to establish custody. It does not initiate peer fetch; DA evidence may independently cause Marshal backfill. `fetch_block` is the explicit network retrieval API and can return buffered bytes before storage sync; a fetch response alone does not establish custody. A wrong peer response is not proof that the expected payload itself is permanently invalid. Local storage failure is also distinct from invalid peer content.

The diagram shows one eligible live validation request. `verify` also runs locally before signing and during recovery; qualify role/lane, lifecycle and exact identity before treating success as a new peer candidate. Nonproducing validators have no own-lane exclusion, while Observers normally enter through Update. [Role and origin rules](../02-components/04-app-coordination.md#when-speculative-execution-begins).

Admission uses the App owner's current state after custody completes. Required producer ancestry and authentication still apply, and the scheduler waits for the exact execution checkpoint before dispatch. Already-applied or retired speculative work can still require an honest native custody response. Follow the [candidate/dispatch handler](../02-components/05-app-callbacks.md#automatonverify-validity-custody-and-scheduling) for deferred-capacity, duplicate and stale-request handling.

Transactions retire through durable canonical outcomes, not body publication, `verify(true)`, or selection. See [TxPool](../02-components/06-tx-pool.md) for internal backend reconciliation and [canonical delivery](06-ordered-application.md) for the later Update path.
