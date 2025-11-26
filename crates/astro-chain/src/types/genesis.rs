use crate::PublicKey;
use commonware_codec::DecodeExt;
use commonware_utils::from_hex_formatted;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genesis {
    /// List of all validators at genesis block
    pub validators: Vec<Validator>,
    /// Amount of time to wait for a leader to propose a payload
    /// in a view.
    pub leader_timeout_ms: u64,
    /// Amount of time to wait for a quorum of notarizations in a view
    /// before attempting to skip the view.
    pub notarization_timeout_ms: u64,
    /// Amount of time to wait before retrying a nullify broadcast if
    /// stuck in a view.
    pub nullify_timeout_ms: u64,
    /// Number of views behind finalized tip to track
    /// and persist activity derived from validator messages.
    pub activity_timeout_views: u64,
    /// Move to nullify immediately if the selected leader has been inactive
    /// for this many views.
    ///
    /// This number should be less than or equal to `activity_timeout` (how
    /// many views we are tracking).
    pub skip_timeout_views: u64,
    /// Maximum size allowed for messages over any connection.
    ///
    /// The actual size of the network message will be higher due to overhead from the protocol;
    /// this may include additional metadata, data from the codec, and/or cryptographic signatures.
    pub max_message_size_bytes: u64,
    /// Prefix for all signed messages to prevent replay attacks.
    pub namespace: String,
    /// network polynomial identity
    pub identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub public_key: String,
    pub ip_address: String,
}

impl Validator {
    /// Get the raw PublicKey from the hex-formatted public_key string
    pub fn get_raw_pubkey(&self) -> Result<PublicKey, String> {
        let pub_key_bytes = from_hex_formatted(&self.public_key).ok_or("PublicKey bad format")?;
        PublicKey::decode(&*pub_key_bytes).map_err(|_| "Unable to decode Public Key".to_string())
    }
}

impl TryInto<(PublicKey, SocketAddr)> for &Validator {
    type Error = String;

    fn try_into(self) -> Result<(PublicKey, SocketAddr), Self::Error> {
        let pub_key_bytes = from_hex_formatted(&self.public_key).ok_or("PublicKey bad format")?;

        Ok((
            PublicKey::decode(&*pub_key_bytes).map_err(|_| "Unable to decode Public Key")?,
            self.ip_address.parse().map_err(|_| "Invalid ip address")?,
        ))
    }
}

impl Genesis {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file_string = std::fs::read_to_string(path)?;
        let genesis: Genesis = toml::from_str(&file_string)?;
        Ok(genesis)
    }

    pub fn get_validator_addresses(
        &self,
    ) -> Result<Vec<(PublicKey, SocketAddr)>, Box<dyn std::error::Error>> {
        let mut validators = Vec::new();

        for validator in &self.validators {
            let public_key_bytes = from_hex_formatted(&validator.public_key)
                .ok_or("Invalid hex format for public key")?;
            let pub_key = PublicKey::decode(&*public_key_bytes)?;
            let socket_addr: SocketAddr = validator.ip_address.parse()?;

            validators.push((pub_key, socket_addr));
        }

        Ok(validators)
    }

    pub fn ip_of(&self, target_public_key: &PublicKey) -> Option<SocketAddr> {
        for validator in &self.validators {
            if let Some(public_key_bytes) = from_hex_formatted(&validator.public_key) {
                if let Ok(pub_key) = PublicKey::decode(&*public_key_bytes) {
                    if &pub_key == target_public_key {
                        if let Ok(socket_addr) = validator.ip_address.parse() {
                            return Some(socket_addr);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_genesis() {
        let genesis = Genesis::load_from_file("../example_genesis.toml").unwrap();
        assert_eq!(genesis.validator_count(), 4);

        let addresses = genesis.get_validator_addresses().unwrap();
        assert_eq!(addresses.len(), 4);
    }

    #[test]
    fn test_validator_lookup() {
        let genesis = Genesis::load_from_file("../example_genesis.toml").unwrap();
        let addresses = genesis.get_validator_addresses().unwrap();

        // Test that we can find the IP for each validator
        for (pub_key, expected_addr) in &addresses {
            let found_addr = genesis.ip_of(pub_key);
            assert_eq!(found_addr, Some(*expected_addr));
        }
    }

    #[test]
    fn test_validator_get_raw_pubkey() {
        // Test error case with invalid hex characters
        let invalid_validator = Validator {
            public_key: "invalid_hex_characters".to_string(),
            ip_address: "127.0.0.1:8080".to_string(),
        };

        let result = invalid_validator.get_raw_pubkey();
        assert!(result.is_err(), "Should fail with invalid hex");
        assert!(
            result.unwrap_err().contains("PublicKey bad format"),
            "Error should mention bad format"
        );

        // Test error case with valid hex but wrong length (too short)
        let wrong_length_validator = Validator {
            public_key: "abcdef".to_string(), // Too short for ed25519
            ip_address: "127.0.0.1:8080".to_string(),
        };

        let result = wrong_length_validator.get_raw_pubkey();
        assert!(result.is_err(), "Should fail with wrong length hex");

        // Test that the method exists and can be called (basic smoke test)
        // Using a 32-byte hex string (64 hex characters)
        let test_hex = "0000000000000000000000000000000000000000000000000000000000000000";
        let validator = Validator {
            public_key: test_hex.to_string(),
            ip_address: "127.0.0.1:8080".to_string(),
        };

        // The method should run without panicking (result may be Ok or Err depending on validity)
        let _result = validator.get_raw_pubkey();
        // Note: We don't assert on success here since all-zero might not be a valid ed25519 key
        // but we're testing that the method works and doesn't panic
    }
}
