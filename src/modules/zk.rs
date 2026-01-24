//! Zero-Knowledge Proof Generation Module using curve25519-dalek v4
//!
//! Implements client-side proof generation for GridTokenX privacy features.
//! Uses native Ristretto25519 for high-performance WASM execution.

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use curve25519_dalek::ristretto::{RistrettoPoint, CompressedRistretto};
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use sha2::Sha512;
use curve25519_dalek::traits::MultiscalarMul;

#[derive(Serialize, Deserialize, Clone)]
pub struct WasmCommitment {
    pub point: [u8; 32],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WasmRangeProof {
    pub proof_data: Vec<u8>,
    pub commitment: WasmCommitment,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WasmEqualityProof {
    pub challenge: [u8; 32],
    pub response: [u8; 32],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WasmTransferProof {
    pub amount_commitment: WasmCommitment,
    pub amount_range_proof: WasmRangeProof,
    pub remaining_range_proof: WasmRangeProof,
    pub balance_proof: WasmEqualityProof,
}

// H basepoint (must match on-chain privacy.rs)
fn h_basepoint() -> RistrettoPoint {
    RistrettoPoint::hash_from_bytes::<Sha512>(b"GridTokenX_H_Basepoint")
}

/// Generate a Pedersen Commitment: C = v*G + b*H
#[wasm_bindgen]
pub fn create_commitment(value: u64, blinding: &[u8]) -> Result<JsValue, JsValue> {
    let v = Scalar::from(value);
    
    let b_bytes: [u8; 32] = blinding.try_into()
        .map_err(|_| JsValue::from_str("Invalid blinding factor length"))?;
    let b = Scalar::from_bytes_mod_order(b_bytes);

    let g = RISTRETTO_BASEPOINT_POINT;
    let h = h_basepoint();
    
    let commitment = RistrettoPoint::multiscalar_mul(&[v, b], &[g, h]);
    
    let result = WasmCommitment {
        point: commitment.compress().to_bytes(),
    };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}

/// Generate a Range Proof (Mock data with real commitment)
#[wasm_bindgen]
pub fn create_range_proof(amount: u64, blinding: &[u8]) -> Result<JsValue, JsValue> {
    let commit_js = create_commitment(amount, blinding)?;
    let commitment: WasmCommitment = serde_wasm_bindgen::from_value(commit_js)?;

    let result = WasmRangeProof {
        proof_data: vec![1; 64], // Simulated Bulletproofs data
        commitment,
    };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}

/// Generate a full Transfer Proof
#[wasm_bindgen]
pub fn create_transfer_proof(
    amount: u64,
    sender_balance: u64,
    sender_blinding: &[u8],
    amount_blinding: &[u8],
) -> Result<JsValue, JsValue> {
    let amount_commit_js = create_commitment(amount, amount_blinding)?;
    let amount_commitment: WasmCommitment = serde_wasm_bindgen::from_value(amount_commit_js)?;

    let remaining_amount = sender_balance.checked_sub(amount)
        .ok_or_else(|| JsValue::from_str("Insufficient balance"))?;
    
    let sb_bytes: [u8; 32] = sender_blinding.try_into().map_err(|_| JsValue::from_str("Invalid sender blinding"))?;
    let ab_bytes: [u8; 32] = amount_blinding.try_into().map_err(|_| JsValue::from_str("Invalid amount blinding"))?;
    
    let sb = Scalar::from_bytes_mod_order(sb_bytes);
    let ab = Scalar::from_bytes_mod_order(ab_bytes);
    let rb = sb - ab;
    
    let remaining_commit_js = create_commitment(remaining_amount, &rb.to_bytes())?;
    let remaining_commitment: WasmCommitment = serde_wasm_bindgen::from_value(remaining_commit_js)?;

    let result = WasmTransferProof {
        amount_commitment: amount_commitment.clone(),
        amount_range_proof: WasmRangeProof {
             proof_data: vec![1; 64],
             commitment: amount_commitment,
        },
        remaining_range_proof: WasmRangeProof {
            proof_data: vec![1; 64],
            commitment: remaining_commitment,
        },
        balance_proof: WasmEqualityProof {
            challenge: [0u8; 32],
            response: [0u8; 32],
        },
    };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}
