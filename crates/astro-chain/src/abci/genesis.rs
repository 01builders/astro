use std::{fs, path::PathBuf};

use crate::abci::executor::AbciExecutorError;
use crate::{types::Validator as AstroValidator, AbciExecutor};
use chrono::DateTime;
use serde::Deserialize;

pub(super) struct GenesisHandler {
    home_directory: PathBuf,
}

impl GenesisHandler {
    pub(super) fn new(home_directory: PathBuf) -> Self {
        Self { home_directory }
    }
}

impl GenesisHandler {
    pub(crate) async fn do_genesis(
        &mut self,
        executor: &mut AbciExecutor,
    ) -> Result<[u8; 32], AbciExecutorError> {
        let genesis_path = self.home_directory.join("config/genesis.json");
        let contents = fs::read_to_string(&genesis_path)
            .map_err(|e| AbciExecutorError::Status(tonic::Status::internal(format!("read genesis: {e}"))))?;

        let doc: GenesisDoc = serde_json::from_str(&contents)
            .map_err(|e| AbciExecutorError::Status(tonic::Status::internal(format!("parse genesis: {e}"))))?;

        let genesis_time_unix_ms = if doc.genesis_time.is_empty() {
            0
        } else {
            DateTime::parse_from_rfc3339(&doc.genesis_time)
                .map(|dt| dt.timestamp_millis().max(0) as u64)
                .unwrap_or(0)
        };
        let initial_height = doc.initial_height.max(0) as u64;
        let chain_id = doc.chain_id;
        let app_state_bytes = serde_json::to_vec(&doc.app_state)
            .map_err(|e| AbciExecutorError::Status(tonic::Status::internal(format!("encode app_state: {e}"))))?;

        // Validator entry is optional; executor currently ignores this placeholder.
        let placeholder_validator = AstroValidator {
            public_key: String::new(),
            ip_address: String::new(),
        };

        executor
            .do_genesis(
                genesis_time_unix_ms,
                initial_height,
                chain_id,
                app_state_bytes,
                placeholder_validator,
            )
            .await
    }
}

#[derive(Debug, Deserialize)]
struct GenesisDoc {
    #[serde(default)]
    genesis_time: String,
    chain_id: String,
    #[serde(default)]
    initial_height: i64,
    #[serde(default)]
    app_state: serde_json::Value,
}
