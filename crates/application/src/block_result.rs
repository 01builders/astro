use astro_proto_types::cometbft::abci::v1beta2::Event;
use astro_proto_types::cometbft::abci::v1beta3::{ExecTxResult};

pub struct BlockResult {
    pub app_hash: [u8; 32],
    pub events: Vec<Event>,
    pub tx_results: Vec<ExecTxResult>,
}