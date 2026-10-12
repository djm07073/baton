# Certified state import

<a id="state-sync-from-certified-execution-results"></a>

**An App may import verified peer material during normal execution.** It verifies the result certificate and exact input context, checks applicable material, then fences conflicting work and applies through the same canonical writer used for direct execution.

## Safe import at a glance

```mermaid
flowchart TB
    P["Peer result"] --> V["Execution<br/>[A] results.accept"]
    V --> M["Storage<br/>[P] Storage::verify_material"]
    M -->|"verified and retained"| C["[P] Storage::commit(verified)<br/>single canonical writer"]
    C -->|"[P] Committed"| I["[A] execution.install_checkpoint<br/>skip covered work, retire conflicts"]
    C -->|"[P] Committed"| A["Execution owner ACKs covered Updates"]
    V -->|"invalid / unresolved"| W["No import commit<br/>valid direct work can continue"]
    M -->|"missing / mismatch"| W
```

[Open full-size diagram](../assets/diagrams/diagram-41.svg) · [PNG](../assets/diagrams/diagram-41.png)

Checkpoint installation adds no extra ACK round. Applying imported material and direct effects share writer fencing and exact predecessor checks.

Read after [certified checkpoint handoff](../../execution/README.md#certification-state-sync-from-certified-execution-results). This case follows material verification, `Storage::commit`, `Committed` and Execution checkpoint installation through timing, completion and failure paths.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../03-reference/01-communication-contracts.md#diagram-notation) for contracts and implementation ownership.

## State sync from certified execution results

Execution verifies the import certificate and requests commit after Storage verifies and retains the matching material. Storage owns material transfer and persistence; Execution receives its usable durable checkpoint and skips the covered local computation. Peer result messages do not pass through either scheduler mode for approval. The path shares the single writer, access fencing and durability contract of [canonical application](06-ordered-application.md).

<details>
<summary>Detailed interface sequence</summary>

```mermaid
sequenceDiagram
    participant P as Peer Validator App
    participant A as Local Execution owner
    participant S as Storage
    participant E as Execution / execution workers
    participant M as Multimmit Marshal
    Note over A,E: Local Execution is at A, certified target covers ABC
    P-->>E: wire: ExecutionCertificate(statement, signatures)
    E->>E: [A] results.accept(certificate, canonical_context) -> verified or pending
    alt Invalid or unresolved irrevocable input / base
        Note over E: Reject or retain pending evidence, continue valid direct work
    else VerifiedCertificate for exact input and base
        E-->>P: wire: MaterialRequest(verified_target)
        P-->>E: wire: StateMaterial(target, data, outputs)
        E->>S: [P] Storage::verify_material(verified, material, base)
        S-->>E: PreparedImport or unavailable / invalid
        alt Material not applicable
            Note over A: Continue direct work, no authority transfer
        else Verified applicable PreparedImport
            E->>S: [P] Storage::commit(verified)
            S-->>E: Ok(()), commit accepted, not durable
            Note over E,S: Execution verified f+1, Storage matches material and rechecks base before mutation
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

</details>

Direct and imported state application both require a matching verified `f+1` certificate. Alignment records ordering independently. A certificate alone does not justify stopping unfinished execution; applicable material must also be verified. Do not apply a canonical-base delta to arbitrary partial speculative state. A verified import can satisfy an outstanding Update only when its exact input and durable applied identity cover that Update in order. Do not synthesize ACK tokens or skip unrelated outstanding delivery.

For checkpoint catch-up, coordinate Marshal's authenticated floor and output identity with App state installation. `install_floor` restores Marshal's input prefix, not the application database. Serialize this App-owned transition with canonical intake and fence old workers; reconcile already retained Updates against the exact durable imported prefix before resuming. A Marshal floor reset may open a new delivery window while App still holds old Updates, so either prevent that overlap through the transition lifecycle or bound it explicitly. `Update` has no generation tag; local import ownership and exact identity checks must supply the fence. Wire/material formats, checkpoint switching, result verification and atomic recovery linkage remain implementation work. [State-sync responsibilities](../../execution/README.md#certification-state-sync-from-certified-execution-results)
