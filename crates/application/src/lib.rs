use commonware_cryptography::bls12381::primitives::group;
use commonware_cryptography::bls12381::primitives::poly::Poly;
use astro_types::{Evaluation, PublicKey};

mod ingress;
mod abci_executor;
mod actor;
mod genesis;
mod propose;
mod utils;
mod supervisor;
mod block_result;

/// Configuration for the application.
pub struct Config {
    /// Participants active in consensus.
    pub participants: Vec<PublicKey>,

    /// The unevaluated group polynomial associated with the current dealing.
    pub polynomial: Poly<Evaluation>,

    /// The share of the secret.
    pub share: group::Share,

    /// Number of messages from consensus to hold in our backlog
    /// before blocking.
    pub mailbox_size: usize,

    /// The underlying ABCI client endpoint.
    pub abci_app_endpoint: String,
}