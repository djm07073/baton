# Summary

* [Start here](README.md)

## 01. Overview

* [Architecture](01-overview/01-architecture.md)
* [Roles and terminology](01-overview/02-terminology.md)

## 02. Components: responsibilities and communication

* [Multimmit: native consensus](02-components/01-multimmit.md)
* [Marshal: block custody and exchange](02-components/02-marshal-custody.md)
* [Marshal: ordered delivery and ACK](02-components/03-marshal-delivery.md)
* [App: coordination and ownership](02-components/04-app-coordination.md)
* [App: native callbacks](02-components/05-app-callbacks.md)
* [TxPool: admission and selection](02-components/06-tx-pool.md)
* [Alignment: Pre-cut and Baton](02-components/07-alignment.md)
* [Executor: execution and checkpoints](02-components/08-executor.md)
* [Storage: reads and durable state](02-components/09-storage.md)
* [Result processing: signing and certification](02-components/10-result-processing.md)
* [State sync: certified checkpoint handoff](02-components/11-state-sync.md)
* [P2P: channels and messages](02-components/12-networking.md)

## 03. E2E flows

* [Transaction to durable state](03-e2e/01-transaction-to-state.md)
* [Block publication and retrieval](03-e2e/02-block-exchange.md)
* [Native proposal, DA and finality](03-e2e/03-native-consensus.md)
* [Leader reports and proposal races](03-e2e/04-leader-reports.md)
* [Direction receipt and execution](03-e2e/05-direction-and-execution.md)
* [Ordered input to durable application](03-e2e/06-ordered-application.md)
* [Execution result certification](03-e2e/07-result-certification.md)
* [Certified state import](03-e2e/08-state-import.md)
* [Startup, restart and recovery](03-e2e/09-startup-and-recovery.md)

## 04. Reference and development

* [Communication contracts and type details](04-reference/01-communication-contracts.md)
* [Existing Rust signatures](04-reference/02-rust-signatures.md)
* [QMDB branches and persistence](04-reference/03-qmdb.md)
* [Pool backend survey](04-reference/04-pool-backends.md)
* [Native integration open decisions](04-reference/05-native-open-decisions.md)
* [Pre-cut baseline and comparisons](04-reference/06-precut-baseline.md)
* [Commonware integration and development order](04-reference/07-development-order.md)
* [Cases to verify](04-reference/08-verification.md)
* [Sources and document status](04-reference/09-sources.md)
