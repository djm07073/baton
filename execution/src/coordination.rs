//! Execution ownership and the existing native callback interfaces.

use crate::{
    execution::Worker,
    storage::{Storage, StorageEvent},
    types::CanonicalInput,
};
use baton_alignment::AlignmentPolicy;
use baton_tx_pool::{TxPool, payload::Payload};
use commonware_actor::Feedback;
use commonware_consensus::{
    Automaton, Reporter,
    multimmit::{
        marshal,
        types::{Context, TransactionBlock},
    },
};
use commonware_cryptography::{Sha256, bls12381::primitives::variant::MinPk, sha256::Digest};
use commonware_utils::{channel::oneshot, sync::Mutex};
use std::{collections::VecDeque, sync::Arc};

/// Full producer block using the transaction-pool payload codec.
pub type Block = TransactionBlock<Sha256, Payload>;

/// Existing Marshal handle. The reference hasher/variant follow log-multimmit.
/// Native key selection does not choose the separate f+1 result-signing scheme.
pub type Marshal = marshal::Mailbox<Sha256, MinPk, Payload>;

/// Cloneable native callback handle reaching one shared owner.
/// Spec: README.md#coordination
pub struct Execution<S: Storage, A: AlignmentPolicy> {
    owner: Arc<Mutex<Owner<S, A>>>,
}

impl<S: Storage, A: AlignmentPolicy> Clone for Execution<S, A> {
    fn clone(&self) -> Self {
        Self {
            owner: Arc::clone(&self.owner),
        }
    }
}

struct Owner<S: Storage, A: AlignmentPolicy> {
    marshal: Option<Marshal>,
    alignment: A,
    pool: TxPool,
    execution: Worker,
    storage: S,
    updates: VecDeque<marshal::Update<Block>>,
    // CALLBACK-01: Add lifecycle, retained events and bounded dispatch claims.
}

impl<S: Storage, A: AlignmentPolicy> Execution<S, A> {
    /// Creates retained Update ingress before Marshal starts and returns its mailbox.
    pub fn open(_pool: TxPool, _alignment: A, _storage: S) -> Self {
        // CALLBACK-01: Wire one owner and its bounded queues; do not activate speculation here.
        // Spec: README.md#callback-01
        todo!("CALLBACK-01: construct callback handles and recovering Execution ownership");
    }
    /// Attaches custody access and starts callback work before native Engine recovery.
    pub fn start(&self, _marshal: Marshal) {
        // CALLBACK-01: Attach once; retain early Updates and keep speculation gated by readiness.
        // Spec: README.md#coordination-assemble-before-opening-the-engine
        todo!("CALLBACK-01: attach the returned Marshal mailbox and start supervised workers");
    }
    #[expect(
        clippy::unused_async,
        reason = "APPLY-01: remove when the worker awaits execution and submit"
    )]
    pub(crate) async fn apply_ordered(&self, _input: CanonicalInput) {
        // APPLY-01: Worker awaits whole-unit reconcile and Storage::submit acceptance before the next unit.
        // Use short owner messages/lock scopes; never hold the Execution lock across an await.
        // Spec: README.md#apply-01
        todo!("APPLY-01: drive retained canonical work through accepted storage submission");
    }
}

impl<S: Storage + 'static, A: AlignmentPolicy> Automaton for Execution<S, A> {
    type Context = Context<Digest>;
    type Digest = Digest;

    async fn propose(&mut self, _context: Context<Digest>) -> oneshot::Receiver<Digest> {
        // CALLBACK-01: Return the receiver promptly; bounded work selects, builds and stages the full block.
        // Send its body digest after staging acceptance; retain bytes for local custody verification.
        // Spec: README.md#coordination-automatonpropose-pool-to-staged-block
        todo!("CALLBACK-01: implement native propose intake and the staged-body response");
    }
    async fn verify(
        &mut self,
        _context: Context<Digest>,
        _body: Digest,
    ) -> oneshot::Receiver<bool> {
        // CALLBACK-01: Validate exact body/context and durable custody, then optionally admit a candidate.
        // Keep temporary gaps pending; never await execution, roots or f+1 for this verdict.
        // Spec: README.md#coordination-automatonverify-validity-custody-and-scheduling
        todo!("CALLBACK-01: implement native verification, cancellation and lifecycle checks");
    }
}

impl<S: Storage + 'static, A: AlignmentPolicy> Reporter for Execution<S, A> {
    type Activity = marshal::Update<Block>;

    fn report(&mut self, _update: Self::Activity) -> Feedback {
        // APPLY-01: Retain the complete Update/Exact token and reflect Alignment ordering synchronously.
        // Feedback is intake completion; Backoff cannot recover a dropped Update.
        // Spec: README.md#coordination-marshal-reportupdate-canonical-input-and-ack
        todo!("APPLY-01: retain native ordered delivery without execution or ACK waits");
    }
}

impl<S: Storage, A: AlignmentPolicy> Owner<S, A> {
    fn dispatch(&mut self) {
        // CALLBACK-01: Recheck AlignmentPolicy::next, claim exact attempt/resources and launch a runtime worker.
        // Spec: README.md#coordination-dispatch-after-verification
        todo!("CALLBACK-01: implement claimed asynchronous dispatch and completion fencing");
    }
    fn on_storage(&mut self, _event: StorageEvent) {
        // APPLY-01: Prepared goes to Execution certification; Committed publishes checkpoint and covered ACKs.
        // Failed keeps ACKs withheld; mutable-instance failure enters recovery.
        // Spec: README.md#apply-01
        todo!("APPLY-01: route retained events, checkpoint installation and exact durable ACKs");
    }
}
