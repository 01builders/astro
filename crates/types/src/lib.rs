mod block;
pub mod genesis;
mod consensus;

pub use block::*;
use commonware_cryptography::bls12381::primitives::variant::{MinPk, Variant};
pub use genesis::*;
pub use consensus::*;

use commonware_consensus::threshold_simplex::types::Activity as CActivity;

pub type Digest = commonware_cryptography::sha256::Digest;
pub type Activity = CActivity<MinPk, Digest>;

pub type PublicKey = commonware_cryptography::ed25519::PublicKey;
pub type PrivateKey = commonware_cryptography::ed25519::PrivateKey;
pub type Signature = commonware_cryptography::ed25519::Signature;
pub type Identity = <MinPk as Variant>::Public;

/// Network namespace for Astro protocol
pub const NAMESPACE: &[u8] = b"astro";
