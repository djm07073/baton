# Leader reports and proposal races

<a id="leader-reports-direction-and-proposal-races"></a>

**Baton is one scheduler mode inside the App.** It gathers signed intended orders and prepares advisory direction while native consensus proceeds. The sequences explain policy behavior and its interaction with consensus; private methods and worker decomposition are implementation choices.

## Planning and cut proceed independently

```mermaid
flowchart TB
    R["Valid intended-order reports"] --> S["Close snapshot once<br/>first 4f+1 or fixed deadline"]
    S --> E["Finish bounded candidate evaluation<br/>2f+1 entire-prefix support, else sum-LCP"]
    E --> D["Prepared advisory direction"]
    C["Native cut opportunity"] --> Q{"Valid policy ready<br/>for actual context?"}
    D -.->|"[X] authenticated adoption hook"| Q
    Q -->|"Yes"| P["[X] Freeze and preserve policy<br/>in authenticated proposal"]
    Q -->|"No, before adoption"| B["Use actual-parent valid base<br/>without waiting"]
```

[Open full-size diagram](../assets/diagrams/diagram-37.svg) · [PNG](../assets/diagrams/diagram-37.png)

The policy choice uses sum-LCP only with a nonempty frozen report snapshot. No reports or no completed valid candidate uses the base path before adoption; an adopted protected prefix cannot be discarded as fallback.

Read after [Alignment policy](../02-components/06-alignment.md). This case follows report/direction messages and the explicitly unresolved native policy hooks through timing, completion and failure paths.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../04-reference/01-communication-contracts.md#diagram-notation) for contracts and implementation ownership.

## Leader lifecycle: collect reports and disseminate direction

External callers use `Alignment::admit/commit`; shared internal dispatch uses `AlignmentPolicy::next`. Report collection and direction selection below are behavior owned by `BatonAlignment`, not additional SDK methods to implement. The diagram does not prescribe private function names or separate planning workers.

The report protocol below remains application work. Native leader-window context and authenticated proposal-policy adoption additionally require the [unfinished native extension](../04-reference/05-native-open-decisions.md). Producer `Automaton::propose` and `verify` are not leader-cut callbacks.

<details>
<summary>Detailed interface sequence</summary>

```mermaid
sequenceDiagram
    participant N as Native proposal owner
    participant L as Leader BatonAlignment
    participant V as Validator BatonAlignment
    par App report window
        N-->>L: [X] leader_context() -> LeaderContext
        L->>L: [A] Fix window context and deadline
        L-->>V: wire: WindowAnnouncement(context, deadline)
        loop Valid reports admitted before closure
            V->>V: [A] Snapshot intended order from F and pending
            V-->>L: wire: SignedReport(context, intended_order)
            L->>L: [A] Authenticate and retain distinct report
        end
        L->>L: [A] Freeze snapshot at threshold or deadline
        alt Empty frozen snapshot
            Note over L,N: No prepared direction, valid native base before adoption
        else Nonempty FrozenReports
            L->>L: [A] Evaluate directions within the fixed budget
            opt Current, valid PreparedDirection
                L-->>V: wire: SignedDirection(context, order)
                V->>V: [A] Validate direction and permitted pending changes
            end
        end
    and Independent native cut
        N->>N: Native construction and voting
        Note over N,L: Never await report count, deadline, planning or direction ACK
    end
    Note over L,V: Internal processing, no additional SDK methods
    Note over L,V: Wire schemas are App design placeholders, no Ready quorum
```

[Open full-size diagram](../assets/diagrams/diagram-06.svg)

</details>

A report is the fixed completed/current local prefix `F` followed by `sort_G(S_pending)`. It is an intention, not an execution proof. Equal context, `F`, rule and pending set produce equal intended order; equal total known inputs alone do not. Later arrivals do not change an already signed report or a closed snapshot.

The leader counts at most `4f+1` distinct valid original reports. It closes on the first threshold/deadline event processed, without extending the deadline. With a nonempty snapshot and fully evaluated bounded admissible candidate set, select the longest nonempty entire prefix with at least `2f+1` original same-context supporters; if no such prefix exists, use raw sum-LCP. Before protected adoption, no reports or no prepared valid choice means the actual-parent valid native base path. Incomplete search does not establish a longest-prefix result.

<a id="planner-completion-versus-proposal-freeze"></a>

## Planning completion versus proposal freeze

This is the **required future integration flow**, not an API present in current Multimmit:

<details>
<summary>Detailed interface sequence</summary>

```mermaid
sequenceDiagram
    participant A as BatonAlignment
    participant N as Native proposal owner
    participant V as Native validators
    A->>A: [A] Evaluate frozen reports within budget
    alt Evaluation completes before proposal freeze
        opt Still-current prepared candidate
            A-->>N: [X] offer_policy(prepared)
        end
    else Empty snapshot, incomplete, stale or unavailable planning
        Note over A,N: No valid prepared candidate, native cut never awaits this work
    end
    N->>N: [X] adopt_policy(actual_context, ready_candidate)
    alt Valid candidate and adoption conditions hold
        Note over N: Freeze prefix / policy in authenticated proposal subject
    else No valid policy before adoption
        Note over N: Use valid actual-parent native base
    end
    N-->>V: wire: LeaderBlock plus future authenticated policy encoding
    V->>V: [X] validate_policy(proposal, actual_context)
    V-->>N: wire: Native votes only after native / policy validation
    opt Later report or planning completion
        Note over A: Keep the result bound to its original context
        Note over N,V: Already adopted interpretation cannot change or revert to base
    end
    Note over N,V: X calls are missing design obligations, not existing callable APIs
```

[Open full-size diagram](../assets/diagrams/diagram-12.svg)

</details>

Current `LeaderBlock` has no Baton policy field, and current Marshal uses the native order interpretation. Adding scheduling in `Automaton::verify` does not implement policy adoption, validator policy checks, exact leading-prefix preservation, continuation or recovery. The App cannot implement this by reordering finalized Updates after delivery.

Before adoption, no reports or no prepared candidate permits the valid native base path. After authenticated adoption, the protected prefix cannot silently disappear or be reinterpreted as fallback. Context binding, availability/adoption, native sufficiency, cross-lane continuation and view recovery remain open proof and implementation obligations. The local report window closing is distinct from that still-unspecified protected-adoption point.
