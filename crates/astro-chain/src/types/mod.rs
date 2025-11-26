mod block;
mod consensus;
pub(crate) mod genesis;
mod block_result;

pub use block::*;
pub use block_result::BlockResult;
pub use consensus::*;
pub use genesis::*;

use commonware_consensus::threshold_simplex::types::Activity as CActivity;
use commonware_cryptography::bls12381::primitives::variant::{MinPk, Variant};

pub type Digest = commonware_cryptography::sha256::Digest;
pub type Activity = CActivity<MinPk, Digest>;

pub type PublicKey = commonware_cryptography::ed25519::PublicKey;
pub type PrivateKey = commonware_cryptography::ed25519::PrivateKey;
pub type Signature = commonware_cryptography::ed25519::Signature;
pub type Identity = <MinPk as Variant>::Public;

/// Network namespace for Astro protocol
pub const NAMESPACE: &[u8] = b"astro";
