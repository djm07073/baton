# Native proposal, DA and finality

<a id="block-proposal-da-and-cut-finality"></a>

**Multimmit owns native signing, voting and finality.** The App provides payloads and validity/custody verdicts. Marshal consumes native activity to retain and deliver the ordinary finalized total order.

<a id="normal-native-consensus-producer-da--leader-proposal--finality"></a>

Read after [Multimmit](../02-components/01-multimmit.md) and [Marshal delivery](../02-components/03-marshal-delivery.md). This case follows Automaton, native Activity and the final Update handoff through timing, completion and failure paths.

Diagram labels: **[C]** existing Commonware interface; **[A]** App-owned processing; **[P]** proposed App trait; **[X]** required native extension; **wire** peer message. See the [interface map](../04-reference/01-communication-contracts.md#diagram-notation) for contracts and implementation ownership.

## Native producer, DA and view/finality paths

This diagram highlights the App connections. Native protocol details remain in the [consensus source map](../02-components/01-multimmit.md).

```mermaid
sequenceDiagram
    participant P as Producer native owner
    participant A as Producer App
    participant V as Validator native owner
    participant B as Validator App
    participant L as Native view / finality owner
    participant M as Multimmit Marshal
    P->>A: [C] Automaton::propose(ctx).await
    A-->>P: Receiver[BodyDigest], later digest_reply.send(body_digest)
    P->>A: [C] Automaton::verify(ctx, body_digest).await
    A-->>P: Receiver[bool], later verdict_reply.send(true)
    P->>P: Existing local custody fence and header signing
    P-->>V: wire: SignedTransactionBlock(header, attestation)
    V->>V: Native authentication and eligibility
    V->>B: [C] Automaton::verify(ctx, body_digest).await
    B-->>V: Receiver[bool], later true after valid durable custody
    par App speculation
        B->>B: [A] app.admit_candidate(candidate)
        B->>B: [A] app.dispatch_ready() when exact parent / resources ready
    and Native DA
        V->>V: Existing DA and native persistence checks
        V-->>P: wire: DA vote
        P-->>V: wire: DA certificate at native threshold
    and Native view / finality
        Note over P,L: DA certificate assembly and leader progress are independent, native eligibility still applies
        L-->>V: wire: LeaderBlock and required parent evidence
        V-->>L: wire: Native votes to peers including leader
        L->>L: Existing finality and extension processing
        L->>M: [C] Reporter::report(Activity)
        Note over M: Existing history / body / order reconstruction feeds App Update reporter
    end
```

[Open full-size diagram](../assets/diagrams/diagram-15.svg)

The App's speculative execution is independent of the native DA path after successful validity/custody verification. Neither speculative worker completion nor application delivery ACK is a native vote/cut prerequisite. Native availability, ancestry and durability gates still apply; the body validity callback is not permission to bypass them.

Native view work can already be running during producer validation. The leader/view-owner lifeline represents one native owner; each view owner processes finality locally.

The native data message carries the signed producer header. Marshal separately broadcasts the full `TransactionBlock` containing header and body. A receiver can therefore have a header while still waiting for its body. Verification begins when native eligibility chooses the request, not for every packet arrival. [Body flow](02-block-exchange.md)

Native activity is also useful for App observations through a composed `Reporters` value. It is distinct from Marshal `Update`: the former reports native activity, while the latter supplies an indexed finalized complete block and application ACK token. The [current assembly](https://github.com/0xEyrie/monorepo/blob/6233438985d8249d2b2bc1204191d5d405652288/examples/log-multimmit/src/node.rs#L306) passes native activities to Marshal and an App handle.

For Baton to change the agreed execution order, a separate native proposal-policy extension must preserve selected leading prefixes through validation, extension and recovery. Current App scheduling callbacks alone do not implement that extension. [Open native decisions](../04-reference/05-native-open-decisions.md)
