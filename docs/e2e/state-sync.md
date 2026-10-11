# Certified state import

<a id="state-sync-from-certified-execution-results"></a>

**An App may import verified peer material during normal execution.** It verifies the result certificate and exact input context, checks applicable material, then fences conflicting work and applies through the same canonical writer used for direct execution.

Read after [certified checkpoint handoff](../execution/state-sync.md). This case follows material verification, `Storage::certify`, `Committed` and Executor checkpoint installation through timing, completion and failure paths.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../overview/interfaces.md#diagram-notation) for contracts and implementation ownership.

## State sync from certified execution results

State sync belongs to Storage. Executor receives its usable durable checkpoint and skips the covered local computation. Peer result messages do not pass through either scheduler mode for approval. The path shares the single writer, access fencing and durability contract of [canonical application](canonical.md).

```mermaid
sequenceDiagram
    participant P as Peer Validator App
    participant A as Local App owner
    participant S as Storage
    participant E as App Executor
    participant M as Multimmit Marshal
    Note over A,E: Local Executor is at A, certified target covers ABC
    P-->>A: wire: ExecutionCertificate(statement, signatures)
    A->>A: [A] results.accept(certificate, canonical_context)
    alt Invalid or unresolved irrevocable input / base
        Note over A: Reject or retain pending evidence, continue valid direct work
    else VerifiedCertificate for exact input and base
        A-->>P: wire: MaterialRequest(verified_target)
        P-->>A: wire: StateMaterial(target, data, outputs)
        A->>S: [A] state.verify_material(certificate, material, base)
        S-->>A: PreparedImport or unavailable / invalid
        alt Material not applicable
            Note over A: Continue direct work, no authority transfer
        else Verified applicable PreparedImport
            A->>S: [P] Storage::certify(certificate)
            S-->>A: Ok(()), admitted for verification, not durable
            Note over A,S: Verify f+1 and material, recheck base and fence conflicts before mutation
            alt Rejected before mutation
                S-->>A: [P] StorageEvent::Failed(failure), no ACK
                Note over A,S: Retain input, reconcile context, no ACK
            else Successful durability and recoverable certificate linkage
                S-->>A: [P] StorageEvent::Committed(applied), checkpoint ABC, imported
                A->>A: [A] app.publish_applied(applied)
                A->>E: [A] execution.install_checkpoint(applied.checkpoint)
                E->>E: Skip B/C, retire covered or conflicting attempts
                Note over A,E: Next job uses ABC as parent, stale completions cannot attach
                opt Retained Updates covered by exact durable identity
                    A-->>M: [C] update.acknowledgement.acknowledge()
                end
                A->>A: [A] app.dispatch_ready() from imported base
            else Mutation or durability failure
                Note over A,S: No ACK, stop failed store and recover authoritative state
            end
        end
    end
    Note over P,A: No own direct-execution signature for imported input
```

[Open full-size diagram](../assets/diagrams/diagram-14.svg)

Direct and imported state application both require a matching verified `f+1` certificate. Alignment records ordering independently. A certificate alone does not justify stopping unfinished execution; applicable material must also be verified. Do not apply a canonical-base delta to arbitrary partial speculative state. A verified import can satisfy an outstanding Update only when its exact input and durable applied identity cover that Update in order. Do not synthesize ACK tokens or skip unrelated outstanding delivery.

For checkpoint catch-up, coordinate Marshal's authenticated floor and output identity with App state installation. `install_floor` restores Marshal's input prefix, not the application database. Serialize this App-owned transition with canonical intake and fence old workers; reconcile already retained Updates against the exact durable imported prefix before resuming. A Marshal floor reset may open a new delivery window while App still holds old Updates, so either prevent that overlap through the transition lifecycle or bound it explicitly. `Update` has no generation tag; local import ownership and exact identity checks must supply the fence. Wire/material formats, checkpoint switching, result verification and atomic recovery linkage remain implementation work. [State-sync responsibilities](../execution/state-sync.md)
