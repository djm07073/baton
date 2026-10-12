# Direction receipt and execution

<a id="receiving-direction-and-executing-branches"></a>

**The App scheduler decides pending work; Execution execution code runs it from an exact parent.** Pre-cut and Baton modes share those execution and storage mechanisms. Baton additionally receives authenticated advisory direction. Public `execute` or `reschedule` traits are unnecessary for this internal connection.

## Only pending work is reordered

```mermaid
flowchart TB
    A["A is running<br/>eligible C arrives, then B"] --> Q{"Has C started on A?"}
    Q -->|"No"| P["F = A<br/>sort pending to B → C"]
    P --> ABC["After exact parents complete<br/>execute A → B → C"]
    Q -->|"Yes"| F["F = AC<br/>keep started order, B pending"]
    F --> ACB["If ancestry permits<br/>execute A → C → B"]
    O["Later authoritative Marshal order"] --> R["[A] execution.reconcile<br/>reuse exact prefix, repair suffix"]
    ABC -.-> R
    ACB -.-> R
```

[Open full-size diagram](../assets/diagrams/diagram-38.svg) · [PNG](../assets/diagrams/diagram-38.png)

This example assumes `G = A → B → C`; the actual comparator remains open. Arrival alone does not undo started work. Canonical ordering can still require repair; an advisory direction’s precedence over a conflicting started path remains open.

Read after [Alignment](../../alignment/README.md#alignment) and [Execution](../../execution/README.md#execution). This case follows `AlignmentPolicy::next`, the App claim step and `execution.execute` through timing, completion and failure paths.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../03-reference/01-communication-contracts.md#diagram-notation) for contracts and implementation ownership.

## Local execution: A executing, C then B arrive

For a shared global rule A → B → C, preserve A while it executes. When eligible C and then B arrive before C starts, sort only the pending queue to B → C. Dispatch B on A's completed checkpoint, then C on AB. Admission while A runs changes pending order but does not start dependent work from an incomplete state.

If C already started on A, late eligible B does not move it: the local path remains AC, followed by B if ancestry permits. Do not wait for unknown B or reexecute C merely to restore a fresh global sort. The common rule is `F ++ sort_G(S_pending)`, with the started/completed valid prefix `F` held fixed. Reports use the same intention, and already signed reports remain immutable.

A later continuous finalized Update may require another order. The App then finds the matching canonical prefix and repairs only the required suffix. Local started order does not establish finality. [Scheduling contract](../../alignment/README.md#alignment)

<a id="non-leader-lifecycle-direction-and-rescheduling"></a>

## Non-leader lifecycle: direction and branch execution

<details>
<summary>Detailed interface sequence</summary>

```mermaid
sequenceDiagram
    participant L as Peer BatonAlignment
    participant B as Local BatonAlignment
    participant A as Alignment internal dispatch / Execution owner
    participant E as Execution / execution workers
    L-->>B: wire: SignedDirection(context, intended_order)
    B->>B: [A] Validate direction context and pending-order change
    alt Invalid or stale
        Note over B: Reject without changing valid current work
    else Started-prefix conflict with no adopted override policy
        Note over B: Defer, the conflict policy remains open
    else Permitted pending-order change
        B-->>A: [A] Coalesced dispatch wakeup, no payload
        A->>A: [A] app.dispatch_ready()
        A->>B: [P] AlignmentPolicy::next(current_view), cross-crate
        B-->>A: JobIntent or None
        alt None
            Note over A: Wait for eligible input, ready parent or capacity
        else JobIntent available
            A->>A: Recheck current view, claim attempt and reserve resources
            alt Claim failed or intent stale
                Note over A: Refresh view, do not launch
            else Claimed Job recorded before launch
                A->>E: [A] execution.execute(job)
                E-->>A: ExecutionOutcome(job_id, StateChanges)
                A->>A: [A] app.finish_job(job_id, outcome)
            end
        end
    end
    Note over B,A: Both are Alignment internals, external calls use Alignment::admit / commit
    Note over A,E: Missing blocks follow the body/candidate flow before becoming eligible
```

[Open full-size diagram](../assets/diagrams/diagram-07.svg)

</details>

The diagram starts at direction reception; blocks must independently pass the [body validation/custody flow](02-block-exchange.md) and candidate admission before `AlignmentPolicy::next` can select them. The same dispatch loop uses that cross-crate trait for PreCut. A direction does not make missing bytes or an unfinished parent usable.

The precedence of a conflicting advisory direction over an already started local path remains undecided. A valid signature alone does not authorize rolling back canonical state or changing a frozen native proposal policy. Preserve that open decision rather than implementing a hidden rescheduling default in the diagram.

Execution reuse requires identical canonical base, ordered input prefix and runtime, with a completed valid checkpoint. A producer header parent is only the previous block in its own lane; it is not this cross-lane execution parent. [Exact execution and storage responsibilities](../../execution/README.md#execution)

The Execution owner accepts a completion only once for the still-current execution attempt and its exact context/valid ancestry. Duplicate or retired/canceled attempts cannot advance the path. Dropping a waiter does not prove CPU or storage work stopped; fence affected live database access before canonical mutation and retain references until safe to release.
