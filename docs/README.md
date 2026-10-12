# Baton docs: start here

**Read [01. Architecture](01-overview/01-architecture.md) first.** It shows how Network assembles the existing Multimmit Engine/Marshal with Execution, TxPool, Alignment and Storage. Execution implements the native application callbacks and owns coordination.

For implementation, start with [developer assignments](03-reference/07-development-order.md#developer-assignments). Source `todo!()` markers identify owned work, required behavior and completion tests; the scaffold does not implement those contracts yet.

## Choose a reading path

```mermaid
flowchart LR
    A["01 · Overview<br/>Who owns what?"] --> B["Crate READMEs<br/>What crosses each interface?"]
    B --> C["02 · E2E<br/>What happens next?"]
    B -.-> D["03 · Reference<br/>Exact contracts and open work"]
    C -.-> D
```

[Open full-size diagram](assets/diagrams/diagram-22.svg) · [PNG](assets/diagrams/diagram-22.png)

Start with the picture on each page. E2E pages show a short path first; expand their detailed sequences when you need exact calls and failure handling. **[C]** is existing Commonware, **[P]** a proposed App contract, **[A]** App processing, and **[X]** unfinished native work. Value/ownership maps describe data relationships rather than new interfaces.

## Reading order

Start with `01-overview`, read the crate READMEs, then follow `02-e2e`. Keep `03-reference` nearby for exact contracts and implementation assignments. Component details live with their source crate, without another copy in this directory.

1. **`01-overview/` — understand the whole composition.** Read [Architecture](01-overview/01-architecture.md), then [terminology](01-overview/02-terminology.md) for the values passed between components.
2. **Crate READMEs — understand each responsibility and exchange.** Start with [Multimmit](../network/README.md#multimmit) and follow the [component contents](SUMMARY.md): communication partners, inputs, outputs and completion conditions.
3. **`02-e2e/` — follow the complete flow.** Start with [Transaction to durable state](02-e2e/01-transaction-to-state.md). Files `02`–`09` expand individual paths and recovery.
4. **`03-reference/` — look up details as needed.** Consult [communication contracts](03-reference/01-communication-contracts.md), Rust signatures, backend research and development checks from the component and E2E links.

Read [Network](../network/README.md), [TxPool](../tx-pool/README.md), [Alignment](../alignment/README.md), [Execution](../execution/README.md) and [Storage](../storage/README.md) for their responsibilities and communication contracts. Once these responsibilities and exchanges are clear, continue to the first E2E page. Detailed type declarations can wait until you need them.

The [full contents](SUMMARY.md) and site's previous/next links use this same order. `assets/` contains diagram renders and the generated Rust export; it is not a reading chapter.

## Before following E2E

Keep three completion boundaries in mind:

- `verify(true)` establishes payload validity and custody; eligible execution is dispatched independently.
- `Alignment::commit(ordered)` records finalized ordering immediately. `Storage::submit` accepts executed changes; `Storage::commit` accepts an Execution-verified request for persistence.
- `StorageEvent::Committed(applied)` establishes certified local durability, supplies Execution's next usable checkpoint and permits ACK of the covered Updates. Execution installation adds no ACK round.

## Document status

These pages describe a proposed App connected to native source at `6233438985d8249d2b2bc1204191d5d405652288`. App implementation, protected-prefix Baton native integration, E2E execution and performance results remain incomplete. Pre-cut and Baton share the App execution/storage contracts; full Baton additionally needs authenticated native policy adoption, preservation and recovery. Backend, comparator, wire and resource policies remain open. The Bank application model remains outside scope.

Markdown is canonical; the site and Rust export are generated from it. See [sources and document status](03-reference/09-sources.md) for evidence and historical material.
