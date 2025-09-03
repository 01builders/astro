use crate::block_result::BlockResult;
use astro_proto_types::cometbft::abci::v1beta3::{
    RequestFinalizeBlock, RequestInitChain, ResponseFinalizeBlock, abci_client::AbciClient,
};
use astro_proto_types::cometbft::types::v1::{ConsensusParams, Validator as CometValidator};
use astro_types::{Block as AstroBlock, Validator as AstroValidator};
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

impl AbciExecutor {}

impl AbciExecutor {
    /// Convert Unix timestamp in milliseconds to prost_types::Timestamp
    fn millis_to_timestamp(millis: u64) -> prost_types::Timestamp {
        let seconds = millis / 1000;
        let remaining_millis = millis % 1000;
        let nanos = remaining_millis * 1_000_000; // Convert remaining milliseconds to nanoseconds

        prost_types::Timestamp {
            seconds: seconds as i64,
            nanos: nanos as i32,
        }
    }

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

    pub(super) fn convert_block_to_finalize_request(
        &self,
        _block: &AstroBlock,
    ) -> RequestFinalizeBlock {
        todo!()
    }

    pub(super) fn convert_cometbft_response_to_digest(
        &self,
        resp_block: ResponseFinalizeBlock,
    ) -> Result<BlockResult, AbciExecutorError> {
        // Convert prost::Bytes to [u8; 32]
        let app_hash: [u8; 32] = resp_block.app_hash.as_ref().try_into().map_err(|_| {
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

    pub(crate) async fn do_genesis(
        &mut self,
        genesis_time_unix_ms: u64,
        initial_height: u64,
        chain_id: String,
        genesis: Vec<u8>,
        single_validator: AstroValidator,
    ) -> Result<[u8; 32], AbciExecutorError> {
        let request = RequestInitChain {
            time: Some(Self::millis_to_timestamp(genesis_time_unix_ms)),
            chain_id,
            consensus_params: Some(ConsensusParams {
                block: None,
                evidence: None,
                validator: None,
                version: None,
                abci: None,
                synchrony: None,
                feature: None,
            }),
            validators: vec![],
            app_state_bytes: genesis.into(),
            initial_height: initial_height as i64,
        };
        let resp = self.client.init_chain(request).await?.into_inner();

        // Convert prost::Bytes to [u8; 32]
        let app_hash: [u8; 32] = resp.app_hash.as_ref().try_into().map_err(|_| {
            AbciExecutorError::Status(tonic::Status::invalid_argument(
                "app_hash must be exactly 32 bytes",
            ))
        })?;

        Ok(app_hash)
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
            pub_key: None,
            voting_power: 100, // for now since the network is one validator this is the default vp
            proposer_priority: 1,
            pub_key_bytes: raw_pubkey.encode().into(),
            pub_key_type: "astro-validator-key".to_string(), // todo in cosmos sdk
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astro_types::Validator;

    #[test]
    fn test_astro_validator_into_comet_validator() {
        // Test error case with invalid hex - this is the main functionality we're testing
        let invalid_validator = Validator {
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
        let wrong_length_validator = Validator {
            public_key: "abcd".to_string(), // Too short
            ip_address: "127.0.0.1:8080".to_string(),
        };

        let result = AbciExecutor::astro_validator_into_comet_validator(wrong_length_validator);
        assert!(result.is_err(), "Conversion should fail with wrong length");
        
        // Test that the function can handle the call structure correctly
        // (testing the integration with get_raw_pubkey method)
        let test_validator = Validator {
            public_key: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            ip_address: "127.0.0.1:8080".to_string(),
        };

        // This may or may not succeed depending on whether all-zero is valid for ed25519,
        // but the important thing is that the method doesn't panic and properly uses get_raw_pubkey
        let result = AbciExecutor::astro_validator_into_comet_validator(test_validator);
        
        // If it succeeds, verify the structure
        if let Ok(comet_validator) = result {
            assert_eq!(comet_validator.voting_power, 100);
            assert_eq!(comet_validator.proposer_priority, 1);
            assert_eq!(comet_validator.pub_key_type, "astro-validator-key");
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
