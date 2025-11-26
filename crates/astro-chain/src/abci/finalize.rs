use crate::types::{Block as AstroBlock, BlockResult};
use commonware_cryptography::Digestible;
use cometbft_proto::tendermint::abci::{
    abci_client::AbciClient, CommitInfo, RequestFinalizeBlock, ResponseFinalizeBlock,
};
use tonic::transport::Channel;
use crate::abci::executor::AbciExecutorError;

pub(crate) async fn finalize_block(
    block: &AstroBlock,
    client: &mut AbciClient<Channel>,
) -> Result<BlockResult, AbciExecutorError> {
    let request_finalize_block = convert_block_to_finalize_request(block);

    let resp: ResponseFinalizeBlock = client
        .finalize_block(request_finalize_block)
        .await?
        .into_inner();

    convert_cometbft_response_to_digest(resp)
}

fn convert_block_to_finalize_request(block: &AstroBlock) -> RequestFinalizeBlock {
    RequestFinalizeBlock {
        txs: block.transactions.clone(),
        decided_last_commit: Some(CommitInfo { round: 0, votes: vec![] }),
        misbehavior: vec![],
        hash: block.digest().to_vec(),
        height: block.height as i64,
        time: Some(super::AbciExecutor::millis_to_timestamp(block.timestamp)),
        next_validators_hash: vec![],
        proposer_address: vec![],
    }
}

fn convert_cometbft_response_to_digest(
    resp_block: ResponseFinalizeBlock,
) -> Result<BlockResult, AbciExecutorError> {
    let app_hash: [u8; 32] = resp_block.app_hash.as_slice().try_into().map_err(|_| {
        AbciExecutorError::Status(tonic::Status::invalid_argument(
            "app_hash must be exactly 32 bytes",
        ))
    })?;

    Ok(BlockResult {
        app_hash,
        events: resp_block.events,
        tx_results: resp_block.tx_results,
    })
}
