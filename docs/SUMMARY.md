# Summary

* [Start here](README.md)

## Overview

* [Architecture](overview/architecture.md)
* [Roles and terminology](overview/glossary.md)

## Components: responsibilities and communication

* [Multimmit: native consensus](consensus/README.md)
* [Marshal: block custody and exchange](consensus/block-body.md)
* [Marshal: ordered delivery and ACK](consensus/ordered-input.md)
* [App: coordination and ownership](baton/README.md)
* [App: native callbacks](baton/interfaces.md)
* [TxPool: admission and selection](tx/interfaces.md)
* [Alignment: Pre-cut and Baton](baton/direction.md)
* [Executor: execution and checkpoints](execution/README.md)
* [Storage: reads and durable state](execution/storage.md)
* [Result processing: signing and certification](execution/interfaces.md)
* [State sync: certified checkpoint handoff](execution/state-sync.md)
* [P2P: channels and messages](overview/networking.md)

## E2E flows

* [Transaction to durable state](e2e/normal.md)
* [Block publication and retrieval](e2e/block-body.md)
* [Native proposal, DA and finality](e2e/native-consensus.md)
* [Leader reports and proposal races](e2e/leader.md)
* [Direction receipt and execution](e2e/reschedule.md)
* [Ordered input to durable application](e2e/canonical.md)
* [Startup, restart and recovery](e2e/recovery.md)
* [Execution result certification](e2e/results.md)
* [Certified state import](e2e/state-sync.md)

## Reference and development

* [Communication contracts and type details](overview/interfaces.md)
* [Existing Rust signatures](overview/rust-interfaces.md)
* [QMDB branches and persistence](execution/qmdb.md)
* [Pool backend survey](tx/README.md)
* [Native integration open decisions](consensus/decisions.md)
* [Pre-cut baseline and comparisons](baselines/precut.md)
* [Commonware integration and development order](reference/integration.md)
* [Cases to verify](reference/verification.md)
* [Sources and document status](reference/sources.md)
