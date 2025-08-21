use crate::block_result::BlockResult;
use astro_proto_types::cometbft::abci::v1beta3::{
    RequestFinalizeBlock, ResponseFinalizeBlock, abci_client::AbciClient,
};
use astro_proto_types::cometbft::abci::v2::InitChainRequest;
use astro_types::Block;
use tonic::transport::Channel;

#[derive(Debug, thiserror::Error)]
pub enum AbciExecutorError {
    #[error("tonic: {0}")]
    Tonic(#[from] tonic::transport::Error),
    #[error("status: {0}")]
    Status(#[from] tonic::Status),
}

pub struct AbciExecutor {
    original_endpoint: String,
    client: AbciClient<Channel>,
}

impl AbciExecutor {}

impl AbciExecutor {
    pub async fn connect(endpoint: &str) -> Result<Self, tonic::transport::Error> {
        let client = AbciClient::connect(endpoint.to_string()).await?;
        Ok(Self::new(client, endpoint.to_string()))
    }

    pub(super) fn new(client: AbciClient<Channel>, original_endpoint: String) -> Self {
        Self {
            client,
            original_endpoint,
        }
    }

    pub(super) async fn finalize_block(
        &mut self,
        block: &Block,
    ) -> Result<BlockResult, AbciExecutorError> {
        let request_finalize_block = self.convert_block_to_finalize_request(block);

        let resp: ResponseFinalizeBlock = self
            .client
            .finalize_block(request_finalize_block)
            .await?
            .into_inner();

        self.convert_cometbft_response_to_digest(resp)
    }

    pub(super) fn convert_block_to_finalize_request(&self, block: &Block) -> RequestFinalizeBlock {
        todo!()
    }

    pub(super) fn convert_cometbft_response_to_digest(
        &self,
        resp_block: ResponseFinalizeBlock,
    ) -> Result<BlockResult, AbciExecutorError> {
        Ok(BlockResult {
            app_hash: resp_block.app_hash.iter().as_slice().try_into()?,
            events: resp_block.events,
            tx_results: resp_block.tx_results,
        })
    }

    pub(crate) async fn do_genesis(
        &mut self,
        genesis: Vec<u8>,
    ) -> Result<[u8; 32], AbciExecutorError> {
        let resp = self.client
            .init_chain(
                InitChainRequest {
                    time: None,
                    chain_id: "".to_string(),
                    consensus_params: None,
                    validators: vec![],
                    app_state_bytes: genesis.into(),
                    initial_height: 0,
                }
                .into(),
            )
            .await?.into_inner();
        
        Ok(resp.app_hash.iter().as_ref().try_into()?)
    }
}
