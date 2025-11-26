use std::{fs, path::PathBuf};

use crate::Digest;
use chrono::DateTime;
use cometbft_proto::google::protobuf::{Duration, Timestamp};
use cometbft_proto::tendermint::{
    abci::{abci_client::AbciClient, RequestInitChain, ValidatorUpdate},
    types as cmt_types,
};
use commonware_utils::from_hex_formatted;
use commonware_cryptography::Hasher;
use commonware_cryptography::Sha256;
use serde::{Deserialize, Deserializer, de::Error as SerdeDeError};
use thiserror::Error;
use tonic::{Status, transport::Channel};

#[derive(Debug, Error)]
pub enum GenesisError {
    #[error("read genesis: {0}")]
    Read(#[from] std::io::Error),
    #[error("parse genesis: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("encode app_state: {0}")]
    AppStateEncode(serde_json::Error),
    #[error("genesis app hash must be empty or zeroed")]
    NonZeroAppHash,
    #[error("genesis validators must be empty, found {0}")]
    NonEmptyValidators(usize),
    #[error("genesis app_state must be present/non-empty")]
    EmptyAppState,
    #[error("genesis app hash length must be 0 or 32 bytes, got {0}")]
    UnexpectedAppHashLength(usize),
    #[error("genesis digest mismatch (expected {expected}, found {found})")]
    GenesisDigestMismatch { expected: String, found: String },
    #[error("init_chain: {0}")]
    AbciStatus(#[from] Status),
    #[error("init_chain app_hash must be 32 bytes, got {0}")]
    InitChainAppHashLength(usize),
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
        _genesis_digest: Digest,
        client: &mut AbciClient<Channel>,
    ) -> Result<([u8; 32], Vec<ValidatorUpdate>), GenesisError> {
        let (doc, raw_bytes) = self.load_genesis_doc()?;
        Self::validate_genesis_doc(&doc, &raw_bytes, _genesis_digest)?;
        let request = Self::genesis_doc_to_request_init_chain(&doc)?;
        let response = client.init_chain(request).await?.into_inner();

        let response_app_hash: [u8; 32] = response
            .app_hash
            .as_slice()
            .try_into()
            .map_err(|_| GenesisError::InitChainAppHashLength(response.app_hash.len()))?;

        Ok((response_app_hash, response.validators))
    }
}

impl GenesisHandler {
    fn load_genesis_doc(&self) -> Result<(GenesisDoc, Vec<u8>), GenesisError> {
        let genesis_path = self.home_directory.join("config/genesis.json");
        let contents = fs::read(&genesis_path)?;
        let doc: GenesisDoc = serde_json::from_slice(&contents)?;
        Ok((doc, contents))
    }

    fn validate_genesis_doc(
        doc: &GenesisDoc,
        raw_bytes: &[u8],
        expected_digest: Digest,
    ) -> Result<(), GenesisError> {
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

        let mut hasher = Sha256::new();
        hasher.update(raw_bytes);
        let found_digest: Digest = hasher.finalize();

        if found_digest != expected_digest {
            return Err(GenesisError::GenesisDigestMismatch {
                expected: to_hex(expected_digest.as_ref()),
                found: to_hex(found_digest.as_ref()),
            });
        }

        Ok(())
    }

    fn genesis_doc_to_request_init_chain(
        doc: &GenesisDoc,
    ) -> Result<RequestInitChain, GenesisError> {
        let genesis_time_unix_ms = if doc.genesis_time.is_empty() {
            0
        } else {
            DateTime::parse_from_rfc3339(&doc.genesis_time)
                .map(|dt| dt.timestamp_millis().max(0) as u64)
                .unwrap_or(0)
        };
        let initial_height = doc.initial_height.max(0) as u64;
        let app_state_bytes =
            serde_json::to_vec(&doc.app_state).map_err(GenesisError::AppStateEncode)?;

        Ok(RequestInitChain {
            time: (genesis_time_unix_ms > 0).then(|| millis_to_timestamp(genesis_time_unix_ms)),
            chain_id: doc.chain_id.clone(),
            consensus_params: doc.consensus_params.as_ref().map(to_proto_consensus_params),
            validators: Vec::new(), // we enforce empty genesis validators above
            app_state_bytes,
            initial_height: initial_height as i64,
        })
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
    #[serde(default, deserialize_with = "deserialize_duration")]
    max_age_duration: DurationValue,
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

#[derive(Debug, Clone, Default)]
struct DurationValue {
    seconds: i64,
    nanos: i32,
}

fn millis_to_timestamp(millis: u64) -> Timestamp {
    let seconds = millis / 1000;
    let nanos = (millis % 1000) * 1_000_000;
    Timestamp {
        seconds: seconds as i64,
        nanos: nanos as i32,
    }
}

fn to_proto_duration(d: &DurationValue) -> Duration {
    Duration { seconds: d.seconds, nanos: d.nanos }
}

fn to_proto_consensus_params(params: &ConsensusParams) -> cmt_types::ConsensusParams {
    cmt_types::ConsensusParams {
        block: params.block.as_ref().map(|b| cmt_types::BlockParams {
            max_bytes: b.max_bytes,
            max_gas: b.max_gas,
        }),
        evidence: params.evidence.as_ref().map(|e| cmt_types::EvidenceParams {
            max_age_num_blocks: e.max_age_num_blocks,
            max_age_duration: Some(to_proto_duration(&e.max_age_duration)),
            max_bytes: e.max_bytes,
        }),
        validator: params
            .validator
            .as_ref()
            .map(|v| cmt_types::ValidatorParams {
            pub_key_types: v.pub_key_types.clone(),
        }),
        version: params
            .version
            .as_ref()
            .map(|v| cmt_types::VersionParams { app: v.app }),
        abci: params.abci.as_ref().map(|a| cmt_types::AbciParams {
            vote_extensions_enable_height: a.vote_extensions_enable_height,
        }),
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

fn deserialize_duration<'de, D>(deserializer: D) -> Result<DurationValue, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Number(num) => num
            .as_i64()
            .or_else(|| num.as_u64().and_then(|v| v.try_into().ok()))
            .map(|seconds| DurationValue { seconds, nanos: 0 })
            .ok_or_else(|| D::Error::custom("invalid integer value for duration")),
        serde_json::Value::String(s) => parse_duration_string(&s)
            .ok_or_else(|| D::Error::custom("invalid duration string")),
        serde_json::Value::Null => Ok(DurationValue { seconds: 0, nanos: 0 }),
        other => Err(D::Error::custom(format!(
            "expected integer, string, or null, got {other:?}"
        ))),
    }
}

fn parse_duration_string(raw: &str) -> Option<DurationValue> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let number_str = trimmed.strip_suffix('s').unwrap_or(trimmed);
    let parsed: f64 = number_str.parse().ok()?;
    let seconds = parsed.trunc() as i64;
    let nanos = ((parsed.fract()) * 1_000_000_000f64).round() as i32;
    Some(DurationValue { seconds, nanos })
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}
