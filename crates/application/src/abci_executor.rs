use tonic::transport::Channel;
use astro_proto_types::cometbft::abci::v1beta3::abci_client::AbciClient;
use astro_types::Block;

pub struct CosmosSDKExecutor {
    client: AbciClient<Channel>,
}

impl CosmosSDKExecutor {
    pub(super) fn new(client: AbciClient<Channel>) -> Self {
        Self { client }
    }

    pub(super) fn execute_block(&self, block: &Block) {

    }
}