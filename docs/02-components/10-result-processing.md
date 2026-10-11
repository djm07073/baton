# Result processing: signing and certification

<a id="app-execution-and-result-certification"></a>

**App result processing signs directly executed results and gathers matching peer evidence.** Executor supplies accepted computation evidence; Storage supplies prepared commitments. Result processing returns certificates to Storage, which independently verifies them before durable application. This is an App responsibility, not another required service or public trait.

## Responsibilities

- Bind the exact ordered input, canonical base, runtime and complete result to the node's direct-execution evidence before signing.
- Authenticate and deduplicate eligible peer signatures over the same full statement.
- Retain the original statements, signatures and serving obligations; distinguish directly executed and imported work.

## Communication interfaces

| Direction | Call, event or value | Completion means |
|---|---|---|
| Executor → App result processing, through the owner | Accepted direct `ExecutionOutcome` and its exact job context | Local computation evidence; it still needs irrevocable input and prepared commitments before signing |
| Storage → App result processing, through the owner | `StorageEvent::Prepared(prepared)` | Commitments/material for the selected unit; no canonical durability yet |
| App result processing → peer Apps | `ExecutionAttestation(statement, signature)` | Own signature created locally for directly executed, fully bound work |
| Peer Apps → App result processing | `ExecutionAttestation` or `ExecutionCertificate` | Local validation yields `VerifiedCertificate`, pending or invalid after full-subject and signer checks |
| App result processing → Storage | `Storage::certify(certificate)` | Evidence accepted for verification; durable completion arrives separately as `StorageEvent::Committed` |

`results.sign(statement, direct_evidence)` and `results.accept(evidence, canonical_context)` are local private helpers behind those exchanges. Peer messages are logical content with open codecs. See the [detailed result contract](../04-reference/01-communication-contracts.md#private-result-import-and-lifecycle-calls) and [message catalog](12-networking.md#messages-used-in-diagrams).

<a id="interface-overview"></a>
<a id="internal-execution-and-apply-flow"></a>

Execution and persistence use the [Executor](08-executor.md) and [Storage](09-storage.md) contracts. The App owner retains each Update and records `Alignment::commit` immediately, then drives independent execution, certification and durability. Follow the [E2E canonical sequence](../03-e2e/06-ordered-application.md) for their timing and the [certification sequence](../03-e2e/07-result-certification.md) for peer exchange. Baton reports and directions never gate this path.

<a id="result-certification-trait"></a>

<a id="result-certification-inside-executor"></a>

## Result certification inside App

App result-message handlers use existing authenticated P2P and crypto directly. Baton report/direction handling does not relay or gate execution certificates, state sync or apply.

App retains the accepted Executor job's direct-computation evidence; Storage emits prepared commitments and serves durable material. Signing and peer collection remain App result handling. Importing a checkpoint can skip local execution, but never manufactures local execution evidence for that imported range.

A direct signer requires its own completed execution/validation evidence, a prepared full result commitment, irrevocable exact input and the correct canonical input state. Bind epoch, range, exact ordered inputs, canonical base, runtime/rule and selected result/output/material commitments. Local worker generations and writer permits are not stable shared signature fields. Advisory speculative order alone does not satisfy the signing condition.

Collect signatures only after validating the full subject, eligible epoch identity and signature. Count each identity once. State finalization requires `f+1` distinct matching signatures plus verified irrevocable order and canonical input-state linkage. That verified certificate is also a prerequisite for canonical state application and the covered Marshal ACKs. Eligible result signers need not belong to the particular ordering QC. Fewer signatures keep application pending while native consensus and valid private execution continue. A root alone or transport response count is insufficient. A certificate for a different result cannot authorize local effects; retain the certified result's matching material or recover the mismatch.

An imported verified result retains imported provenance. The app may relay/serve the original certificate; it cannot issue its own direct-execution signature for work merely imported. It can directly execute and sign a subsequent range from the imported canonical checkpoint. Certification, material availability and local durable/read readiness remain separate milestones. [State sync](11-state-sync.md#state-sync-from-certified-execution-results).

Per-block/per-chunk boundaries, statement codec/domain, result-root scheme, keys, serving/retention and result-switch policy remain open. A longer-range certificate cannot be sliced into a shorter-range certificate.

### Certification units and delivery progress

A certification unit is a bounded continuous ordered range with an exact start base and certified terminal result; its concrete boundaries remain open. All inputs needed to certify the next unit must be obtainable within the active `max_pending_acks` capacity at the participating signers, including after restart or a floor transition. Validate that compatibility during assembly; otherwise waiting for more Updates while withholding every ACK creates a delivery deadlock. Short partial units are valid only if their boundary is part of the common signing policy. Do not solve a full window by ACKing uncertified input.

Within a multi-block unit, compute successive blocks on private exact-parent checkpoints without awaiting per-block canonical application. Prepare and sign the unit's terminal result before the writer applies it; requiring application before signing would create a circular wait. Apply the whole certified unit and then ACK its covered Updates. A terminal certificate cannot be sliced to authorize an intermediate checkpoint unless the authenticated statement explicitly certifies that boundary. This permits bounded range signing without requiring per-block signatures.

Persist the certificate, exact subject and required verification context with the recoverable applied record, or a durable reference whose evidence remains available. On restart, revalidate that linkage before publishing readiness or ACKing replay. Body custody and private work storage may persist earlier; they do not advance the certified canonical applied position. Result certification, canonical durability and Marshal's cursor durability remain distinct sequential conditions.

## Reuse certificate building blocks

| Existing mechanism | Reuse | App check still required |
|---|---|---|
| `p2p::utils::codec` typed wrappers | Encode/decode existing authenticated sender/receiver traffic | Bounded codec, full statement, original eligible signer identities |
| Crypto `Subject`, `Attestation`, `Scheme`, `Signers` | Domain/message definition, signing, signature verification and packaging | Exact input/base/runtime/result interpretation and distinct eligible `f+1` |
| Optional buffered broadcast | Share/cache an identified certificate or material object | Cache key identifies the object; different attestations cannot all overwrite one statement digest |
| Optional collector | Solicit request-bound peer responses | Its count is transport responses before app validation, not valid signer count |

Current source: [certificate APIs](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/cryptography/src/certificate.rs), [typed transport](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/p2p/src/utils/codec.rs), [collector response handling](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/collector/src/p2p/engine.rs#L190).

`Scheme::assemble` packages supplied attestations; it does not replace signature verification. Validate and deduplicate roster indices before constructing signer sets. Decode peer certificates with the selected epoch verifier's bounded `certificate_codec_config`, not its unbounded trusted-storage configuration. Keep a bounded execution-statement envelope because primitive attestation/certificate data does not supply our exact execution context. Scheme-specific proof-of-possession and original-signature provenance requirements still apply.

`N5f1::quorum(n)` computes `n-f`, or `4f+1` for `n=5f+1`; this is not the execution-result threshold. Multimmit also has distinct DA and nullification threshold sharings, not an existing `f+1` result-signing role. Select compatible certificate/key material and quorum handling, or use lower-level signature/bitmap primitives while enforcing `f+1` in app assembly and verification. A quorum adapter cannot lower an existing BLS sharing's polynomial threshold. No new native quorum or result-signing format is adopted. [Current fault model](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/utils/src/faults.rs), [native key roles](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/consensus/src/multimmit/scheme/bls12381_threshold/mod.rs#L3), [BLS threshold compatibility](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/cryptography/src/bls12381/certificate/threshold/mod.rs#L92).
