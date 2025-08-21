use astro_proto_types::cometbft::abci::v2::{Event, TxResult};

pub struct BlockResult {
    pub app_hash: [u8; 32],
    pub events: Vec<Event>,
    pub tx_results: Vec<TxResult>
}