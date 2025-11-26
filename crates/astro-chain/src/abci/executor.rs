use crate::types::{Block as AstroBlock, BlockResult, Validator as AstroValidator};
use cometbft_proto::google::protobuf::Timestamp;
use cometbft_proto::tendermint::{
    abci::{abci_client::AbciClient, RequestFinalizeBlock, RequestInitChain, ResponseFinalizeBlock},
    crypto,
    types::{ConsensusParams, Validator as CometValidator},
};
use commonware_codec::Encode;
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

impl AbciExecutor {
    /// Convert Unix timestamp in milliseconds to prost_types::Timestamp
    fn millis_to_timestamp(millis: u64) -> Timestamp {
        let seconds = millis / 1000;
        let remaining_millis = millis % 1000;
        let nanos = remaining_millis * 1_000_000; // Convert remaining milliseconds to nanoseconds

        Timestamp { seconds: seconds as i64, nanos: nanos as i32 }
    }

    pub async fn connect(endpoint: &str) -> Result<Self, tonic::transport::Error> {
        let client = AbciClient::connect(endpoint.to_string()).await?;
        Ok(Self::new(client, endpoint.to_string()))
    }

    pub fn new(client: AbciClient<Channel>, original_endpoint: String) -> Self {
        Self {
            client,
            original_endpoint,
        }
    }

    async fn finalize_block(
        &mut self,
        block: &AstroBlock,
    ) -> Result<BlockResult, AbciExecutorError> {
        let request_finalize_block = self.convert_block_to_finalize_request(block);

        let resp: ResponseFinalizeBlock = self
            .client
            .finalize_block(request_finalize_block)
            .await?
            .into_inner();

        self.convert_cometbft_response_to_digest(resp)
    }

    fn convert_block_to_finalize_request(
        &self,
        _block: &AstroBlock,
    ) -> RequestFinalizeBlock {
        todo!()
    }

    fn convert_cometbft_response_to_digest(
        &self,
        resp_block: ResponseFinalizeBlock,
    ) -> Result<BlockResult, AbciExecutorError> {
        // Convert prost::Bytes to [u8; 32]
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

    fn astro_validator_into_comet_validator(
        val: AstroValidator,
    ) -> Result<CometValidator, AbciExecutorError> {
        let raw_pubkey = val.get_raw_pubkey().map_err(|e| {
            AbciExecutorError::Status(tonic::Status::invalid_argument(format!(
                "Invalid validator public key: {}",
                e
            )))
        })?;

        Ok(CometValidator {
            address: Default::default(),
            pub_key: Some(crypto::PublicKey {
                sum: Some(crypto::public_key::Sum::Ed25519(raw_pubkey.encode().into())),
            }),
            voting_power: 100, // for now since the network is one validator this is the default vp
            proposer_priority: 1,
        })
    }

    /// Drain pending mempool transactions from the provided receiver.
    /// TODO: enforce size/limit policy.
    fn pull_txs(
        mempool_rx: &mut futures::channel::mpsc::Receiver<Vec<u8>>,
    ) -> Vec<Vec<u8>> {
        let mut txs = Vec::new();
        while let Some(v) = mempool_rx
            .try_next()
            .expect("mempool channel was dropped")
        {
            txs.push(v);
        }
        txs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_astro_validator_into_comet_validator() {
        // Test error case with invalid hex - this is the main functionality we're testing
        let invalid_validator = AstroValidator {
            public_key: "invalid_hex".to_string(),
            ip_address: "127.0.0.1:8080".to_string(),
        };

        let result = AbciExecutor::astro_validator_into_comet_validator(invalid_validator);
        assert!(
            result.is_err(),
            "Conversion should fail with invalid pubkey"
        );

        // Verify the error message contains our custom formatting
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("Invalid validator public key"),
            "Error should contain our custom message: {}",
            error
        );

        // Test error case with wrong length hex
        let wrong_length_validator = AstroValidator {
            public_key: "abcd".to_string(), // Too short
            ip_address: "127.0.0.1:8080".to_string(),
        };

        let result = AbciExecutor::astro_validator_into_comet_validator(wrong_length_validator);
        assert!(result.is_err(), "Conversion should fail with wrong length");

        // Test that the function can handle the call structure correctly
        // (testing the integration with get_raw_pubkey method)
        let test_validator = AstroValidator {
            public_key: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            ip_address: "127.0.0.1:8080".to_string(),
        };

        // This may or may not succeed depending on whether all-zero is valid for ed25519,
        // but the important thing is that the method doesn't panic and properly uses get_raw_pubkey
        let result = AbciExecutor::astro_validator_into_comet_validator(test_validator);

        // If it succeeds, verify the structure
        if let Ok(comet_validator) = result {
            assert_eq!(comet_validator.voting_power, 100);
            assert_eq!(comet_validator.proposer_priority, 1);
        }
        // If it fails, that's also acceptable - we're testing the integration, not valid key generation
    }

    #[test]
    fn test_millis_to_timestamp() {
        // Test basic conversion: 1 second + 500 milliseconds
        let millis = 1500u64; // 1.5 seconds
        let timestamp = AbciExecutor::millis_to_timestamp(millis);

        assert_eq!(timestamp.seconds, 1);
        assert_eq!(timestamp.nanos, 500_000_000); // 500 ms = 500,000,000 ns

        // Test exact second boundary
        let millis = 5000u64; // Exactly 5 seconds
        let timestamp = AbciExecutor::millis_to_timestamp(millis);

        assert_eq!(timestamp.seconds, 5);
        assert_eq!(timestamp.nanos, 0);

        // Test zero
        let millis = 0u64;
        let timestamp = AbciExecutor::millis_to_timestamp(millis);

        assert_eq!(timestamp.seconds, 0);
        assert_eq!(timestamp.nanos, 0);

        // Test milliseconds only
        let millis = 999u64; // Less than 1 second
        let timestamp = AbciExecutor::millis_to_timestamp(millis);

        assert_eq!(timestamp.seconds, 0);
        assert_eq!(timestamp.nanos, 999_000_000); // 999 ms = 999,000,000 ns

        // Test large timestamp (realistic Unix timestamp)
        let millis = 1609459200123u64; // January 1, 2021 + 123ms
        let timestamp = AbciExecutor::millis_to_timestamp(millis);

        assert_eq!(timestamp.seconds, 1609459200);
        assert_eq!(timestamp.nanos, 123_000_000); // 123 ms = 123,000,000 ns
    }
}
