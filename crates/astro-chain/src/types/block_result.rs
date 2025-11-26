use cometbft_proto::tendermint::abci::{Event, ExecTxResult};

pub struct BlockResult {
    pub app_hash: [u8; 32],
    pub events: Vec<Event>,
    pub tx_results: Vec<ExecTxResult>,
}
