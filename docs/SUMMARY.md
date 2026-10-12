# Summary

* [Start here](README.md)

## 01. Overview

* [Architecture](01-overview/01-architecture.md)
* [Roles and terminology](01-overview/02-terminology.md)

## Crates: responsibilities and communication

* [Network: Multimmit, Marshal and assembly](../network/README.md)
* [TxPool: admission, selection and payload](../tx-pool/README.md)
* [Alignment: Pre-cut and Baton](../alignment/README.md)
* [Execution: callbacks, execution and certification](../execution/README.md)
* [Storage: reads and durable state](../storage/README.md)

## 02. E2E flows

* [Transaction to durable state](02-e2e/01-transaction-to-state.md)
* [Block publication and retrieval](02-e2e/02-block-exchange.md)
* [Native proposal, DA and finality](02-e2e/03-native-consensus.md)
* [Leader reports and proposal races](02-e2e/04-leader-reports.md)
* [Direction receipt and execution](02-e2e/05-direction-and-execution.md)
* [Ordered input to durable application](02-e2e/06-ordered-application.md)
* [Execution result certification](02-e2e/07-result-certification.md)
* [Certified state import](02-e2e/08-state-import.md)
* [Startup, restart and recovery](02-e2e/09-startup-and-recovery.md)

## 03. Reference and development

* [Communication contracts and type details](03-reference/01-communication-contracts.md)
* [Existing Rust signatures](03-reference/02-rust-signatures.md)
* [QMDB branches and persistence](03-reference/03-qmdb.md)
* [Pool backend survey](03-reference/04-pool-backends.md)
* [Native integration open decisions](03-reference/05-native-open-decisions.md)
* [Pre-cut baseline and comparisons](03-reference/06-precut-baseline.md)
* [Commonware integration and development order](03-reference/07-development-order.md)
* [Cases to verify](03-reference/08-verification.md)
* [Sources and document status](03-reference/09-sources.md)
