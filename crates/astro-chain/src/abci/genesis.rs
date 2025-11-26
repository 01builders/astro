use std::{fs, path::PathBuf};

use crate::Digest;
use chrono::DateTime;
use cometbft_proto::tendermint::abci::abci_client::AbciClient;
use commonware_utils::from_hex_formatted;
use serde::{Deserialize, Deserializer, de::Error as SerdeDeError};
use thiserror::Error;
use tonic::transport::Channel;

// TODO: ensure we invoke InitChain on the ABCI client using these decoded values.
#[derive(Debug, Error)]
pub enum GenesisError {
    #[error("read genesis: {0}")]
    Read(#[from] std::io::Error),
    #[error("parse genesis: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("genesis app hash must be empty or zeroed")]
    NonZeroAppHash,
    #[error("genesis validators must be empty, found {0}")]
    NonEmptyValidators(usize),
    #[error("genesis app_state must be present/non-empty")]
    EmptyAppState,
    #[error("genesis app hash length must be 0 or 32 bytes, got {0}")]
    UnexpectedAppHashLength(usize),
}

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
        genesis_digest: Digest,
        client: &mut AbciClient<Channel>,
    ) -> Result<[u8; 32], GenesisError> {
        let genesis_path = self.home_directory.join("config/genesis.json");
        let contents = fs::read_to_string(&genesis_path)?;

        let doc: GenesisDoc = serde_json::from_str(&contents)?;

        if !doc.validators.is_empty() {
            return Err(GenesisError::NonEmptyValidators(doc.validators.len()));
        }
        if is_app_state_empty(&doc.app_state) {
            return Err(GenesisError::EmptyAppState);
        }

        if doc.app_hash.is_empty() {
            // allowed, treated as zero hash
        } else if doc.app_hash.len() != 32 {
            return Err(GenesisError::UnexpectedAppHashLength(doc.app_hash.len()));
        } else if doc.app_hash.iter().any(|b| *b != 0) {
            return Err(GenesisError::NonZeroAppHash);
        }

        let app_hash: [u8; 32] = if doc.app_hash.is_empty() {
            [0u8; 32]
        } else {
            doc.app_hash
                .as_slice()
                .try_into()
                .expect("length checked above")
        };

        let genesis_time_unix_ms = if doc.genesis_time.is_empty() {
            0
        } else {
            DateTime::parse_from_rfc3339(&doc.genesis_time)
                .map(|dt| dt.timestamp_millis().max(0) as u64)
                .unwrap_or(0)
        };
        let initial_height = doc.initial_height.max(0) as u64;
        let chain_id = doc.chain_id;
        let app_state_bytes =
            serde_json::to_vec(&doc.app_state).expect("app_state was already validated");

        // TODO: when ABCI wiring is added, use the decoded params above to call InitChain.
        let _ = genesis_digest;
        let _ = client;
        let _ = (
            genesis_time_unix_ms,
            initial_height,
            chain_id,
            app_state_bytes,
        );
        Ok(app_hash)
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
    consensus_params: Option<ConsensusParams>,
    #[serde(default)]
    validators: Vec<GenesisValidator>,
    #[serde(default, deserialize_with = "deserialize_hex_bytes")]
    app_hash: Vec<u8>,
    #[serde(default)]
    app_state: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ConsensusParams {
    #[serde(default)]
    block: Option<BlockParams>,
    #[serde(default)]
    evidence: Option<EvidenceParams>,
    #[serde(default)]
    validator: Option<ValidatorParams>,
    #[serde(default)]
    version: Option<VersionParams>,
    #[serde(default)]
    abci: Option<AbciParams>,
}

#[derive(Debug, Deserialize)]
struct BlockParams {
    #[serde(default, deserialize_with = "deserialize_i64_from_str_or_int")]
    max_bytes: i64,
    #[serde(default, deserialize_with = "deserialize_i64_from_str_or_int")]
    max_gas: i64,
}

#[derive(Debug, Deserialize)]
struct EvidenceParams {
    #[serde(default, deserialize_with = "deserialize_i64_from_str_or_int")]
    max_age_num_blocks: i64,
    #[serde(default, deserialize_with = "deserialize_i64_from_str_or_int")]
    max_age_duration: i64,
    #[serde(default, deserialize_with = "deserialize_i64_from_str_or_int")]
    max_bytes: i64,
}

#[derive(Debug, Deserialize)]
struct ValidatorParams {
    #[serde(default)]
    pub_key_types: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct VersionParams {
    #[serde(default, deserialize_with = "deserialize_u64_from_str_or_int")]
    app: u64,
}

#[derive(Debug, Deserialize)]
struct AbciParams {
    #[serde(default, deserialize_with = "deserialize_i64_from_str_or_int")]
    vote_extensions_enable_height: i64,
}

#[derive(Debug, Deserialize)]
struct GenesisValidator {
    #[serde(default)]
    address: String,
    pub_key: JsonPublicKey,
    #[serde(default, deserialize_with = "deserialize_i64_from_str_or_int")]
    power: i64,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct JsonPublicKey {
    #[serde(rename = "type")]
    key_type: String,
    value: String,
}

fn is_app_state_empty(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::Object(map) if map.is_empty() => true,
        serde_json::Value::Array(arr) if arr.is_empty() => true,
        _ => false,
    }
}

fn deserialize_hex_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;

    if raw.is_empty() {
        return Ok(Vec::new());
    }

    from_hex_formatted(&raw).ok_or_else(|| D::Error::custom("app_hash must be hex-encoded"))
}

fn deserialize_i64_from_str_or_int<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Number(num) => num
            .as_i64()
            .or_else(|| num.as_u64().and_then(|v| v.try_into().ok()))
            .ok_or_else(|| D::Error::custom("invalid integer value")),
        serde_json::Value::String(s) => s
            .parse::<i64>()
            .map_err(|_| SerdeDeError::custom("invalid integer string")),
        serde_json::Value::Null => Ok(0),
        other => Err(D::Error::custom(format!(
            "expected integer or string, got {other:?}"
        ))),
    }
}

fn deserialize_u64_from_str_or_int<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Number(num) => num
            .as_u64()
            .or_else(|| num.as_i64().and_then(|v| v.try_into().ok()))
            .ok_or_else(|| D::Error::custom("invalid integer value")),
        serde_json::Value::String(s) => s
            .parse::<u64>()
            .map_err(|_| SerdeDeError::custom("invalid integer string")),
        serde_json::Value::Null => Ok(0),
        other => Err(D::Error::custom(format!(
            "expected integer or string, got {other:?}"
        ))),
    }
}
