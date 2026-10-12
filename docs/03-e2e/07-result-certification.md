# Execution result certification

<a id="app-result-certification-and-queries"></a>

**Executor creates and verifies execution-result certificates, then requests Storage commit.** Executor checks `f+1` distinct eligible matching signatures for direct and imported results. Storage matches the verified result to its prepared material and persists it. Alignment already reflects finalized ordering through `commit(ordered)` and does not wait for this work. Certification authorizes matching canonical application; successful durability then permits ACK.

## Certificate decision at a glance

```mermaid
flowchart TB
    P["Prepared root + exact context<br/>own direct-execution evidence"] --> S["[A] results.sign<br/>exchange attestations"]
    S --> V{"[A] results.accept"}
    P2["wire: Peer ExecutionCertificate"] --> V
    V -->|"Invalid subject / signer"| R["Reject evidence"]
    V -->|"Context or f+1 missing"| W["Keep pending<br/>no apply or ACK"]
    V -->|"f+1 valid distinct same-subject signatures"| C["[P] Storage::commit(verified)"]
    C --> D["Storage matches material and persists<br/>[P] Committed"]
```

[Open full-size diagram](../assets/diagrams/diagram-40.svg) · [PNG](../assets/diagrams/diagram-40.png)

A received full certificate enters the same verifier without requiring a local signing round. Import additionally needs matching verified material; it never grants direct-signing provenance.

Read after [result processing](../02-components/09-result-processing.md) and [Storage](../02-components/08-storage.md). This case follows `Prepared`, direct signing, peer evidence and `Storage::commit` through timing, completion and failure paths.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../04-reference/01-communication-contracts.md#diagram-notation) for contracts and implementation ownership.

## Result endpoint: direct execution and f+1 certification

The App prepares the selected result commitment before signing. Each speculative attempt need not calculate a root. Result exchange uses Commonware transport and cryptographic primitives, with application-specific exact-context checks; the Baton scheduler does not relay or approve it.

<details>
<summary>Detailed interface sequence</summary>

```mermaid
sequenceDiagram
    participant A as Executor / result processing
    participant S as Storage
    participant P as Peer Validator App
    participant R as Result consumer
    A->>A: Retain StateChanges and accepted direct-job identity
    A->>S: [P] Storage::submit(executed, changes)
    S-->>A: Ok(()), retained but not durable
    S-->>A: [P] StorageEvent::Prepared(prepared)
    A->>A: [A] results.sign(statement, direct_evidence)
    A->>A: Retain original statement and required serving material
    A-->>P: wire: ExecutionAttestation(statement, signature)
    P-->>A: wire: ExecutionAttestation or ExecutionCertificate
    A->>A: [A] results.accept(evidence, canonical_context)
    alt VerifiedCertificate (verified): f+1 distinct eligible same-statement signatures
        A-->>R: wire: ExecutionCertificate(statement, signatures)
        R->>R: Verify received certificate and exact context
        A->>S: [P] Storage::commit(verified)
        S-->>A: Ok(()), commit accepted, not durable
        Note over A,S: Storage matches verified subject and material, batches persistence, then emits Committed
    else Incomplete or context still unresolved
        Note over A: Retain prepared work, no canonical apply or ACK
    else Invalid evidence
        Note over A: Reject, do not count its identity
    end
    Note over A,S: Sign before state application, then commit, apply durably and ACK through the Update owner
    Note over A,P: Imported work cannot produce own direct-execution signature
```

[Open full-size diagram](../assets/diagrams/diagram-11.svg)

</details>

The full subject binds the exact input/range, canonical predecessor state, runtime and result. Equal state roots alone do not make two subjects equal. Each identity counts once under the relevant epoch membership. Honest signers directly execute and validate the subject; relaying an imported certificate cannot create local direct-execution provenance.

The f+1 result threshold is separate from native Multimmit quorums and the 2f+1 intention-prefix support rule. Reports are not result proofs. Result certification does not wait for Baton approval or gate native cut formation. Common signing boundaries, wire encoding, result transport limits and cryptographic scheme remain undecided. [Certification responsibilities](../02-components/09-result-processing.md)

A certified result consumer still distinguishes certificate verification, local readable state and durable applied state. Queries must identify the selected state and result contract. Required outputs and intermediate boundaries must remain available under an explicit retention policy; a root by itself cannot reprovide them. [QMDB and retention](../04-reference/03-qmdb.md)
