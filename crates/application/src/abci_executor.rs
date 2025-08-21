use crate::block_result::BlockResult;
use astro_proto_types::cometbft::abci::v1beta3::{
    RequestFinalizeBlock, ResponseFinalizeBlock, RequestInitChain, abci_client::AbciClient,
};
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

    pub(super) fn convert_block_to_finalize_request(&self, _block: &Block) -> RequestFinalizeBlock {
        todo!()
    }

    pub(super) fn convert_cometbft_response_to_digest(
        &self,
        resp_block: ResponseFinalizeBlock,
    ) -> Result<BlockResult, AbciExecutorError> {
        // Convert prost::Bytes to [u8; 32]
        let app_hash: [u8; 32] = resp_block.app_hash.as_ref().try_into()
            .map_err(|_| AbciExecutorError::Status(tonic::Status::invalid_argument(
                "app_hash must be exactly 32 bytes"
            )))?;
        
        Ok(BlockResult {
            app_hash,
            events: resp_block.events,
            tx_results: resp_block.tx_results,
        })
    }

    pub(crate) async fn do_genesis(
        &mut self,
        genesis: Vec<u8>,
    ) -> Result<[u8; 32], AbciExecutorError> {
        let request = InitChainRequest {
            time: None,
            chain_id: "".to_string(),
            consensus_params: None,
            validators: vec![],
            app_state_bytes: genesis.into(),
            initial_height: 0,
        };
        let resp = self.client
            .init_chain(request)
            .await?.into_inner();
        
        // Convert prost::Bytes to [u8; 32]
        let app_hash: [u8; 32] = resp.app_hash.as_ref().try_into()
            .map_err(|_| AbciExecutorError::Status(tonic::Status::invalid_argument(
                "app_hash must be exactly 32 bytes"
            )))?;
        
        Ok(app_hash)
    }
}
