# Startup, restart and recovery

<a id="startup-restart-and-history-recovery"></a>

**Start body custody and the App callback path before opening native Multimmit.** Native recovery can call `Automaton::verify` from `Engine::open`. Marshal separately restores its finalized output and delivery cursor, while App restores certified durable state, applied identity and certificate evidence. Configured genesis remains the initial trust anchor; every subsequent applied unit needs recoverable certification evidence.

## Two readiness conditions

```mermaid
flowchart TB
    S["App ingress + Marshal custody available"] --> N["[C] Engine::open<br/>may call Automaton::verify"]
    N --> R["[C] engine.start → running.ready"]
    S --> C["[A] state.recover<br/>load candidate + original evidence"]
    C --> V["Executor revalidates evidence<br/>Storage confirms durable record"]
    V --> A["Install recovered checkpoint"]
    R --> G["[A] app.activate<br/>native ready AND App base ready"]
    A --> G
    G --> D["Dispatch work<br/>reconcile / ACK exact replay"]
```

[Open full-size diagram](../assets/diagrams/diagram-42.svg) · [PNG](../assets/diagrams/diagram-42.png)

Invalid recovered evidence stops startup. Fresh configured genesis supplies its trusted checkpoint directly; it does not need an execution certificate.

Read after [App coordination](../02-components/04-app-coordination.md), [Marshal delivery](../02-components/03-marshal-delivery.md) and [Storage](../02-components/09-storage.md). This case follows startup calls, recovered checkpoint handoff and idempotent Update replay through timing, completion and failure paths.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../04-reference/01-communication-contracts.md#diagram-notation) for contracts and implementation ownership.

## Startup: prepare custody before native recovery

For a fresh deployment, initialize the configured deterministic App genesis state/runtime and its initial applied-prefix binding. Marshal output index zero represents stream genesis and is never delivered; the first ordinary Update is index one. Do not wait for an index-zero Update or use a producer genesis-header digest as the App state checkpoint. On restart Storage loads the binding and original certificate evidence, Executor revalidates them, and Storage confirms the matching durable record through idempotent commit before checkpoint installation; checkpoint startup must authenticate the corresponding floor-to-state relation. The actual App genesis contents remain an application choice. [Output index contract](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/consensus/src/multimmit/marshal/types.rs#L55)

<details>
<summary>Detailed interface sequence</summary>

```mermaid
sequenceDiagram
    participant O as Node assembly / owner
    participant W as Existing network
    participant M as Marshal / buffer / resolver
    participant A as Alignment layer / App owner
    participant N as Multimmit Engine
    participant E as App Executor
    participant S as Storage
    O->>W: [C] network.register(channel, quota)
    O->>W: [C] network.start()
    O->>A: [A] app.open(config) -> callback handles
    A->>S: [A] state.recover(config), or configured genesis
    S-->>A: RecoveredState: checkpoint candidate and original evidence
    alt Non-genesis recovered state
        A->>E: [A] results.accept(recovered.evidence, recovered.context)
        E-->>A: VerifiedCertificate or recovery failure
        break Invalid or unresolved recovered evidence
            Note over A,E: Stop startup, no Committed or ACK
        end
        E->>S: [P] Storage::commit(verified), confirm recovered durable record
        S-->>E: Ok(()), matching record accepted
    else Configured genesis
        Note over A,S: Configured trusted initial state, no execution certificate
    end
    S-->>A: [P] StorageEvent::Committed(recovered), checkpoint with pinned StateView
    A->>E: [A] execution.install_checkpoint(recovered.checkpoint)
    Note over O,A: Retained Update ingress exists now, speculative lifecycle stays recovering
    O->>M: [C] buffered::Engine::new(runtime, config)
    M-->>O: broadcast_engine, buffer
    O->>M: [C] broadcast_engine.start(transport.broadcast)
    O->>M: [C] marshal::open(runtime, config, buffer).await
    M-->>O: Service, BackfillBridge
    O->>M: [C] resolver::Engine::new(runtime, config_with_bridge)
    M-->>O: resolver_engine, resolver_mailbox
    O->>M: [C] resolver_engine.start(transport.resolver)
    O->>M: [C] service.relay(local_chain)
    M-->>O: Relay
    O->>M: [C] service.start(resolver_mailbox, verifier, app_output)
    M-->>O: Mailbox, ServiceHandle
    O->>A: [A] app.start(mailbox.clone())
    Note over M,A: Early Updates are retained, custody processing is ready before Engine open
    O->>N: [C] Engine::open(runtime, config).await
    loop Required recovery custody checks
        N->>A: [C] Automaton::verify(ctx, body_digest).await
        A-->>N: Receiver[bool]
        A->>M: [C] subscribe_block(reference).await
        M-->>A: Ok(full_block): durable custody
        A->>A: [A] app.check_payload(ctx, full_block) -> PayloadVerdict
        A-->>N: verdict_reply.send(true) only for valid payload and custody
    end
    N-->>O: Opened Engine or open error
    alt Open failed
        Note over O,N: Stop startup attempt
    else Engine opened
        O->>N: [C] engine.start(planes)
        N-->>O: Running
        O->>N: [C] running.ready().await
        alt Stopped
            Note over O,N: Stop startup attempt
        else Ok and App base consistent
            O->>A: [A] app.activate(native_ready, recovered_base)
            A->>A: [A] app.dispatch_ready()
        end
    end
    Note over O,N: Failures stop this startup attempt, ready is not ongoing health or App durability
```

[Open full-size diagram](../assets/diagrams/diagram-13.svg)

</details>

Follow the current [node assembly](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/examples/log-multimmit/src/node.rs#L242) and [Marshal assembly](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/examples/log-multimmit/src/marshal.rs#L126). They create buffered broadcast, `Service`, the generic resolver with `BackfillBridge`, the existing Relay and `SchemeVerifier`. Start the App with Marshal access before `Engine::open(...).await`; waiting for native readiness before making body lookup usable would create a recovery dependency cycle.

`Engine::open` derives the profile, recovers safety state and verifies required recovered payloads, then wires actors. `start(Planes)` spawns them. `Running::ready().await` returns `Result<(), Stopped>` after the native startup milestone; it does not certify App state readiness or remain a live health check. A false or closed recovered verification causes open failure, which is distinct from a stopped running engine. [Engine lifecycle](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/consensus/src/multimmit/engine/mod.rs#L446)

Recovered verify is a custody recheck, not a new peer arrival. Keep an App lifecycle phase or equivalent internal request context so scheduling deduplicates recovered input and skips already applied blocks. The existing `verify(Context, digest)` signature has no caller-origin field; the App must not pretend it can infer every source solely from the arguments. App state-dependent dispatch waits for a valid recovered canonical base; body validity/custody handling can remain available independently where application semantics permit. When the App enters live scheduling, re-evaluate its retained candidate facts against the current applied identity and eligibility instead of waiting for another successful verification callback. The same state-based wakeup handles newly available worker capacity; candidates deliberately discarded under the speculation budget may remain unexecuted until canonical Update.

## Own service lifetimes and durable shutdown

Use Commonware runtime task ownership for network, Marshal and App workers. Stop consumers before their dependencies, preserve pending durable work and release references after workers are fenced. The [example teardown](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/examples/log-multimmit/src/node.rs#L343) aborts and joins Engine and App before stopping Marshal and network; a transaction App must also implement its storage completion and recovery contract.

A task handle, native ready milestone, or cancelled worker does not establish storage durability. Native signing authority, body retention, application writer lifetime and scheduler task lifetime have different obligations. Do not place the canonical writer under an advisory report-window task or use a direction change to revoke recovery material.

## Restart and backfill

<details>
<summary>Detailed interface sequence</summary>

```mermaid
sequenceDiagram
    participant O as Node owner
    participant M as Multimmit Marshal
    participant A as Alignment layer / App owner
    participant E as App Executor
    participant S as Storage
    A->>S: [A] state.recover(config)
    S-->>A: RecoveredState: checkpoint candidate and original evidence
    alt Non-genesis recovered state
        A->>E: [A] results.accept(recovered.evidence, recovered.context)
        E-->>A: VerifiedCertificate or recovery failure
        break Invalid or unresolved recovered evidence
            Note over A,E: Stop startup, no Committed or ACK
        end
        E->>S: [P] Storage::commit(verified), confirm recovered durable record
        S-->>E: Ok(()), matching record accepted
    else Configured genesis
        Note over A,S: Configured trusted initial state, no execution certificate
    end
    S-->>A: [P] StorageEvent::Committed(recovered), exact prefix and pinned StateView
    A->>E: [A] execution.install_checkpoint(recovered.checkpoint)
    Note over A,E: Skip the recovered prefix, do not recreate its execution
    O->>M: [C] marshal::open(...) then Service::start(..., app_output)
    Note over O,M: Existing catalog, custody, order and delivery cursor recovery
    M->>A: [C] Reporter::report(redelivered Update)
    A->>A: [A] app.retain_update(update)
    A->>A: [P] Alignment::commit(ordered), idempotent continuous ordering
    A-->>M: Feedback: retained
    A->>A: [A] app.apply_next()
    alt Exact index / identity covered by certified durable state
        Note over A,S: Recover and verify retained certificate linkage
        A-->>M: [C] update.acknowledgement.acknowledge()
    else Next continuous unapplied input
        A->>E: [A] execution.reconcile(input, recovered_base)
        E-->>A: StateChanges
        A->>S: [P] Storage::submit(executed, changes)
        S-->>A: Ok(()), retained but not durable
        S-->>E: [P] StorageEvent::Prepared(prepared), via App routing
        E->>E: [A] results.sign(statement, direct_evidence)
        Note over E: Exchange peer attestations for the same full subject
        E->>E: [A] results.accept(evidence, canonical_context) -> verified or pending
        Note over E,S: Missing certificate stays pending, no state apply or ACK
        E->>S: [P] Storage::commit(verified)
        S-->>E: Ok(()), commit accepted, not durable
        S->>S: Match verified subject, prepared material and current base, apply durably
        S-->>A: [P] StorageEvent::Committed(applied)
        A->>A: [A] app.publish_applied(applied)
        A->>E: [A] execution.install_checkpoint(applied.checkpoint)
        A-->>M: [C] update.acknowledgement.acknowledge()
    else Identity conflict, gap or apply failure
        Note over A,S: Stop advancement, retain exact input for recovery, no ACK
    end
    M->>M: Existing acknowledged-prefix cursor sync
```

[Open full-size diagram](../assets/diagrams/diagram-10.svg)

</details>

| Progress coordinate | Owner and meaning |
|---|---|
| Native safety journal | Engine safety and signing recovery; does not describe applied transaction state |
| Marshal finalized output/catalog progress | Durable ordinary native ordered blocks and retained recovery evidence |
| Marshal acknowledged cursor | Its durable record of the continuous application-ACK prefix |
| App applied position and identity | Durable application state/output/provenance linkage for exactly applied Updates |

The App may be ahead of Marshal's acknowledged cursor after a crash between apply and cursor sync. Replaying those Updates must verify identity and produce no duplicate effects. A state-root comparison alone is insufficient; output identity and applied position must also be recoverable.

Use Marshal's existing `floor_at`, `install_floor`, `prune` and `progress` operations for its retained history. An installed Marshal floor does not by itself install App execution state; state-sync must bind the imported state to the same output prefix before claiming local readiness. The App owns a serialized floor/import lifecycle that fences old work and reconciles retained old Updates before processing beyond the imported prefix. Marshal can reset its pending window on a floor change while those old Updates remain in App memory; the per-window ACK bound does not bound that overlap. The Update carries no generation field, so the App must use its own transition ownership and exact index/block/applied-state checks. Marshal protects unacknowledged outputs from pruning, but App query, result-serving and worker references may require additional retention. These remain App/storage responsibilities, not a new shared cursor service. [Marshal retention API](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/consensus/src/multimmit/marshal/mailbox.rs#L452)

Baton report windows and advisory work can be rebuilt after recovery. Any future authenticated Baton proposal policy must preserve its original interpretation through native and Marshal recovery; this protocol extension remains [unimplemented](../04-reference/05-native-open-decisions.md).
