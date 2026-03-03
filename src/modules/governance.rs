//! Governance Module
//!
//! WASM-compatible governance client for GridTokenX.
//! Ports the GovernanceProvider.tsx functionality to Rust/WASM.
//!
//! ## Features
//!
//! - Solana blockchain connection via RPC
//! - PoA (Proof of Authority) config fetching
//! - Proposal management (create, vote)
//! - ZK-weighted voting support

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

// ============================================================================
// Types
// ============================================================================

/// Proposal status variants
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Failed,
}

/// A governance proposal
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub support_weight: u64,
    pub oppose_weight: u64,
    pub deadline: i64,
    pub status: ProposalStatus,
    pub has_voted: bool,
}

/// PoA (Proof of Authority) configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoAConfig {
    pub authority: String,
    pub authority_name: String,
    pub contact_info: String,
    pub erc_validation_enabled: bool,
    pub allow_certificate_transfers: bool,
    pub min_energy_amount: u64,
    pub max_erc_amount: u64,
    pub erc_validity_period: u64,
}

/// Governance state containing proposals and config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceState {
    pub proposals: Vec<Proposal>,
    pub poa_config: Option<PoAConfig>,
    pub is_connected: bool,
}

// ============================================================================
// Governance Client
// ============================================================================

/// WASM-exposed governance client
#[wasm_bindgen]
pub struct GovernanceClient {
    rpc_url: String,
    program_id: String,
    state: GovernanceState,
}

#[wasm_bindgen]
impl GovernanceClient {
    /// Create a new governance client
    ///
    /// # Arguments
    /// * `rpc_url` - Solana RPC endpoint (e.g., "http://127.0.0.1:8899")
    /// * `program_id` - Governance program public key
    #[wasm_bindgen(constructor)]
    pub fn new(rpc_url: String, program_id: String) -> Self {
        console_error_panic_hook::set_once();

        let default_proposals = vec![
            Proposal {
                id: "PROP-001".to_string(),
                title: "Increase Solar Feed-in Tariff".to_string(),
                description: "Proposed increase of 12% to the confidential solar feed-in tariff for residential meters.".to_string(),
                support_weight: 45000,
                oppose_weight: 12000,
                deadline: js_sys::Date::now() as i64 + 86400000 * 3,
                status: ProposalStatus::Active,
                has_voted: false,
            },
            Proposal {
                id: "PROP-002".to_string(),
                title: "Deploy Windsor Wind Farm Relay".to_string(),
                description: "Confidential funding for a new private node in the Windsor offshore wind cluster.".to_string(),
                support_weight: 89000,
                oppose_weight: 5000,
                deadline: js_sys::Date::now() as i64 + 86400000 * 5,
                status: ProposalStatus::Active,
                has_voted: false,
            },
        ];

        Self {
            rpc_url,
            program_id,
            state: GovernanceState {
                proposals: default_proposals,
                poa_config: None,
                is_connected: false,
            },
        }
    }

    /// Initialize connection to the blockchain
    /// Returns true if connection successful
    pub fn connect(&mut self) -> bool {
        // In a real implementation, this would:
        // 1. Create a Solana connection
        // 2. Verify the program exists
        // 3. Load the IDL
        // For now, we simulate a successful connection
        self.state.is_connected = true;
        true
    }

    /// Check if connected to the blockchain
    #[wasm_bindgen(getter)]
    pub fn is_connected(&self) -> bool {
        self.state.is_connected
    }

    /// Get current proposals as JSON
    #[wasm_bindgen(getter)]
    pub fn proposals(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.state.proposals)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get PoA config as JSON
    #[wasm_bindgen(getter)]
    pub fn poa_config(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.state.poa_config)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Fetch PoA configuration from the blockchain
    /// This would call the program's poAConfig account
    pub fn fetch_poa_config(&mut self) -> Result<JsValue, JsValue> {
        if !self.state.is_connected {
            return Err(JsValue::from_str("Not connected to blockchain"));
        }

        // In a real implementation, this would:
        // 1. Compute the PDA for poa_config: [b"poa_config"]
        // 2. Fetch the account data from Solana
        // 3. Deserialize using Anchor/Borsh
        // For now, return mock data

        let config = PoAConfig {
            authority: "DksRNiZsEZ3zN8n8ZWfukFqi3z74e5865oZ8wFk38p4X".to_string(),
            authority_name: "GridTokenX PoA Authority".to_string(),
            contact_info: "governance@gridtokenx.xyz".to_string(),
            erc_validation_enabled: true,
            allow_certificate_transfers: false,
            min_energy_amount: 1000,
            max_erc_amount: 1000000,
            erc_validity_period: 86400,
        };

        self.state.poa_config = Some(config.clone());

        serde_wasm_bindgen::to_value(&config)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Cast a private vote on a proposal
    ///
    /// # Arguments
    /// * `proposal_id` - ID of the proposal to vote on
    /// * `support` - true for support, false for oppose
    /// * `private_balance` - ZK private balance amount for weight
    /// * `root_seed` - Privacy root seed for proof generation
    ///
    /// # Returns
    /// Transaction signature or error message
    pub fn vote_private(
        &mut self,
        proposal_id: String,
        support: bool,
        private_balance: u64,
        _root_seed: String,
    ) -> Result<String, JsValue> {
        if !self.state.is_connected {
            return Err(JsValue::from_str("Not connected to blockchain"));
        }

        // In a real implementation, this would:
        // 1. Generate ZK proof of stake weight using root_seed
        // 2. Build the vote transaction
        // 3. Sign and send to Solana
        // For now, we simulate the vote

        let weight = private_balance;

        // Update local proposal state
        if let Some(proposal) = self.state.proposals.iter_mut().find(|p| p.id == proposal_id) {
            if support {
                proposal.support_weight += weight;
            } else {
                proposal.oppose_weight += weight;
            }
            proposal.has_voted = true;
        } else {
            return Err(JsValue::from_str("Proposal not found"));
        }

        // Simulate ZK proof generation delay
        // In real impl: generate_zk_stake_proof(private_balance, root_seed)

        Ok("SIG_VOTE_CONFIDENTIAL_SUCCESS".to_string())
    }

    /// Create a new proposal
    ///
    /// # Arguments
    /// * `title` - Proposal title
    /// * `description` - Proposal description
    ///
    /// # Returns
    /// The new proposal ID
    pub fn create_proposal(&mut self, title: String, description: String) -> Result<String, JsValue> {
        if !self.state.is_connected {
            return Err(JsValue::from_str("Not connected to blockchain"));
        }

        // In a real implementation, this would:
        // 1. Build the create proposal transaction
        // 2. Sign and send to Solana
        // 3. Get the proposal PDA/ID from the response

        let random_num = (js_sys::Math::random() * 1000.0) as u32;
        let new_id = format!("PROP-{:03}", random_num);

        let new_proposal = Proposal {
            id: new_id.clone(),
            title,
            description,
            support_weight: 0,
            oppose_weight: 0,
            deadline: js_sys::Date::now() as i64 + 86400000 * 7,
            status: ProposalStatus::Active,
            has_voted: false,
        };

        self.state.proposals.insert(0, new_proposal);

        Ok(new_id)
    }

    /// Get the full governance state as JSON
    pub fn get_state(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get RPC URL
    #[wasm_bindgen(getter)]
    pub fn rpc_url(&self) -> String {
        self.rpc_url.clone()
    }

    /// Get program ID
    #[wasm_bindgen(getter)]
    pub fn program_id(&self) -> String {
        self.program_id.clone()
    }
}

// ============================================================================
// Standalone Functions
// ============================================================================

/// Decode fixed-size byte array to string (helper for PoA config)
#[wasm_bindgen]
pub fn decode_fixed_string(bytes: &[u8], len: usize) -> String {
    let slice = &bytes[..len.min(bytes.len())];
    String::from_utf8_lossy(slice)
        .replace('\0', "")
        .trim()
        .to_string()
}

/// Compute PDA for PoA config account
#[wasm_bindgen]
pub fn compute_poa_config_pda(program_id: &str) -> Result<String, JsValue> {
    // In real implementation:
    // use solana_program::pubkey::Pubkey;
    // let program_pubkey = Pubkey::from_str(program_id)
    //     .map_err(|e| JsValue::from_str("Invalid program ID"))?;
    // let seeds = [b"poa_config"];
    // let (pda, _) = Pubkey::find_program_address(&seeds, &program_pubkey);
    // Ok(pda.to_string())

    // Mock implementation
    Ok(format!("poa_config_pda_for_{}", program_id))
}

/// Generate ZK proof for stake-weighted voting
#[wasm_bindgen]
pub fn generate_zk_vote_proof(
    balance: u64,
    root_seed: &str,
    proposal_id: &str,
) -> Result<String, JsValue> {
    // In real implementation, this would:
    // 1. Derive keys from root_seed
    // 2. Create commitment to balance
    // 3. Generate ZK proof that balance > 0 without revealing amount
    // For now, return a mock proof

    let proof_input = format!("{}:{}:{}", balance, root_seed, proposal_id);
    let proof_hash = crate::sha256(proof_input.as_bytes());

    Ok(format!("ZK_PROOF_{}", &proof_hash[..32]))
}

/// Verify a ZK vote proof
#[wasm_bindgen]
pub fn verify_zk_vote_proof(proof: &str, proposal_id: &str) -> bool {
    // In real implementation, verify the ZK proof
    proof.starts_with("ZK_PROOF_") && !proposal_id.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_client_new() {
        let client = GovernanceClient::new(
            "http://127.0.0.1:8899".to_string(),
            "DksRNiZsEZ3zN8n8ZWfukFqi3z74e5865oZ8wFk38p4X".to_string(),
        );
        assert!(!client.is_connected());
        assert_eq!(client.rpc_url(), "http://127.0.0.1:8899");
    }

    #[test]
    fn test_governance_client_connect() {
        let mut client = GovernanceClient::new(
            "http://127.0.0.1:8899".to_string(),
            "DksRNiZsEZ3zN8n8ZWfukFqi3z74e5865oZ8wFk38p4X".to_string(),
        );
        assert!(client.connect());
        assert!(client.is_connected());
    }

    #[test]
    fn test_decode_fixed_string() {
        let bytes = b"Hello\0\0\0\0World";
        let result = decode_fixed_string(bytes, 5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_compute_poa_config_pda() {
        let result = compute_poa_config_pda("TestProgramId").unwrap();
        assert!(result.contains("poa_config_pda"));
    }

    #[test]
    fn test_zk_proof_functions() {
        let proof = generate_zk_vote_proof(1000, "seed123", "PROP-001").unwrap();
        assert!(proof.starts_with("ZK_PROOF_"));
        assert!(verify_zk_vote_proof(&proof, "PROP-001"));
    }
}
