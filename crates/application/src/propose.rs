use crate::actor::Actor;
use commonware_runtime::{Clock, Metrics, Spawner};
use rand::Rng;

impl<R: Rng + Spawner + Metrics + Clock> Actor<R> {
    pub(super) async fn pull_txs(&mut self) -> Vec<Vec<u8>> {
        // TODO: implement tx size limit logic
        let mut txs = vec![];

        // poll until we drain;
        while let Some(v) = self
            .mempool_rx
            .try_next()
            .expect("mempool channel was dropped")
        {
            txs.push(v);
        }
        txs
    }
}
