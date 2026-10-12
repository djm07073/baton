//! PAYLOAD-01: bounded transaction-body encoding and digest, not a block service.

use bytes::{BufMut, Bytes};
use commonware_codec::{Buf, EncodeSize, Error, Read, Write};
use commonware_cryptography::{Digestible, sha256::Digest};

/// Transactions selected for one producer payload. The wire encoding is undecided.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Payload {
    pub(crate) transactions: Vec<Bytes>,
}

/// Caller-supplied allocation bounds for untrusted decoding.
#[derive(Clone)]
pub struct PayloadLimits {
    /// Maximum number of transactions accepted from a body.
    pub max_transactions: usize,
    /// Maximum length of one transaction.
    pub max_transaction_bytes: usize,
    /// Maximum cumulative transaction bytes.
    pub max_body_bytes: usize,
}

impl Read for Payload {
    type Cfg = PayloadLimits;

    fn read_cfg(_buf: &mut impl Buf, _cfg: &Self::Cfg) -> Result<Self, Error> {
        // PAYLOAD-01: Bound count and cumulative bytes before allocation; use codec Buf.
        // Spec: README.md#payload-01
        todo!("PAYLOAD-01: implement the agreed bounded canonical payload decoder");
    }
}

impl Write for Payload {
    fn write(&self, _buf: &mut impl BufMut) {
        // PAYLOAD-01: Encode exactly the format accepted by read_cfg; do not adopt a VM format here.
        // Spec: README.md#pool
        todo!("PAYLOAD-01: implement canonical payload encoding");
    }
}

impl EncodeSize for Payload {
    fn encode_size(&self) -> usize {
        // PAYLOAD-01: Match Write exactly, including all length prefixes and overflow checks.
        // Spec: README.md#pool
        todo!("PAYLOAD-01: calculate the agreed encoded length");
    }
}

impl Digestible for Payload {
    type Digest = Digest;

    fn digest(&self) -> Digest {
        // PAYLOAD-01: Bind canonical body bytes to a unique domain; this is not the header digest.
        // Spec: README.md#payload-01
        todo!("PAYLOAD-01: implement the agreed body commitment");
    }
}
