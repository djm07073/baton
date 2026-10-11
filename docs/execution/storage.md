# Storage: reads and durable state

**Storage owns cold state, commitment preparation and certified durable application.** Executor sends state changes; Storage chooses database batches and flush timing. A committed event carries a usable checkpoint back through the App owner to Executor.

## Responsibilities

| Responsibility | Owned state or work |
|---|---|
| Serve exact-base reads | Retained read-only views backed by pinned versions or access fences |
| Retain ordered execution progress | Bounded StateChanges bound to exact input, predecessor, runtime and accepted job identity |
| Prepare commitments | Deterministic material and roots for the agreed signing unit |
| Apply certified state | Independent `f+1`, order, material and predecessor checks under one canonical writer |
| Recover and import | Linked state, outputs, applied identity, certificate evidence and direct/imported provenance |
| Publish durable progress | Retained events and a usable canonical checkpoint for App consumers |

Storage owns database handles and physical retention. App coordination retains Marshal Updates and their ACK tokens; App result processing supplies signatures/certificates; Executor owns transaction computation and memory checkpoints.

## Communication interfaces

| Direction | Call or event | Completion means |
|---|---|---|
| Executor → Storage | `Storage::read(view, key)` | `Ok(Some(value))` or `Ok(None)` at the exact pinned base; failure is `Err(StorageError)` |
| Executor → App owner → Storage | `Storage::submit(executed, changes)` | `Ok(())` accepts and retains ordered work; it may still be memory-only |
| Storage → App owner / result processing | `StorageEvent::Prepared(prepared)` | Exact commitments ready to combine with direct-execution evidence for signing |
| App result processing → Storage | `Storage::certify(certificate)` | `Ok(())` accepts evidence for verification; it does not establish persistence |
| Storage → App owner | `StorageEvent::Committed(applied)` | The covered certified prefix and its evidence are recoverable; `applied.checkpoint` supplies usable state access |
| Storage → App owner | `StorageEvent::Failed(failure)` | A scoped asynchronous failure; no ACK or unsupported progress follows |

The [canonical Storage contract](../overview/interfaces.md#proposed-storage-interface) specifies the proposed trait, logical types, errors and event retention. Calls and later events have separate completion conditions. Concrete startup/import helpers remain private integration; they do not add methods to this trait.

## Reads and accepted changes

Executor checks memory and parent overlays first. A tombstone resolves deletion locally; only a genuine miss reads the pinned Storage view. A pruned, unavailable or conflicting view must fail rather than answer from current state.

The App owner binds changes to an exact bounded ordered range. Storage checks the predecessor chain and can retain successive submitted units before earlier units flush. Rejection or backpressure leaves the caller responsible for retryable changes; after acceptance Storage owns their retention. On crash, only recovered certified durable progress is authoritative, and lost accepted work is replayed or reexecuted.

## Preparation, certification and durability

Storage materializes exact changes and prepares roots for the chosen unit. Result processing may sign only with matching direct-execution evidence and irrevocable input. Storage independently verifies the eventual certificate's full subject, eligible distinct `f+1` signers, current predecessor and available material before the canonical writer applies it.

Physical persistence may batch successive authorized units while preserving signed roots, operation history and unit boundaries. Flush policy must drain within delivery-window, idle and shutdown constraints. Missing certification keeps prepared work pending. Mutable-store or durability failure stops that store instance until recovery.

Persist state, outputs, applied position, certificate evidence and provenance with recoverable linkage before emitting `Committed`. A readable root or completed QMDB write alone does not establish that linkage. [QMDB operations](qmdb.md#commit-a-branch-to-canonical-state) are the backend reference.

## Checkpoint handoff

The App owner retains `applied.checkpoint` and forwards it through `execution.install_checkpoint`. Executor installs the compatible base, skips covered work, retires conflicts and preserves valid descendants. Memory/views stay retained until workers and references release them. The owner ACKs exactly covered Updates after certified durability; installation and pool maintenance add no approval round.

Verified [state import](state-sync.md) and recovery use this same writer and retained checkpoint handoff. Imported work cannot become local direct-signing evidence. A recovered certified applied record supports idempotent redelivery without reapplying effects or collecting a new certificate. Configured genesis supplies the initial trust anchor.

Continue with [result processing](interfaces.md) to see where the certificate comes from. Once the components are familiar, follow [ordered input to durable application](../e2e/canonical.md), [state import](../e2e/state-sync.md) and [recovery](../e2e/recovery.md) as E2E cases.
