use std::io;
use commonware_runtime::{Clock, Metrics, Spawner};
use rand::Rng;

use crate::actor::Actor;

impl<R: Rng + Spawner + Metrics + Clock> Actor<R> {
    pub(super) async fn fetch_genesis_file(&self) -> Result<Vec<u8>, io::Error> {
        // todo fetch from config
        todo!("impl")
    }
}