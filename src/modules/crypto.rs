//! Cryptographic Operations Module
//! 
//! Signing, verification, and hashing for P2P trade messages.
//! Uses standard Rust crates wrapped for WASM.

use wasm_bindgen::prelude::*;
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};

// ============================================================================
// SHA-256
// ============================================================================

/// Compute SHA-256 hash of input bytes
/// Returns hex string
#[wasm_bindgen]
pub fn sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

// ============================================================================
// HMAC-SHA256
// ============================================================================

/// Compute HMAC-SHA256
/// Returns hex string
#[wasm_bindgen]
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> Result<String, JsValue> {
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| JsValue::from_str("Invalid key length"))?;
    
    mac.update(message);
    let result = mac.finalize().into_bytes();
    Ok(hex::encode(result))
}

/// Verify an HMAC-SHA256 signature
#[wasm_bindgen]
pub fn crypto_verify(key: &[u8], message: &[u8], signature_hex: &str) -> bool {
    // Decode hex signature
    let signature_bytes = match hex::decode(signature_hex) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = match HmacSha256::new_from_slice(key) {
        Ok(m) => m,
        Err(_) => return false,
    };
    
    mac.update(message);
    mac.verify_slice(&signature_bytes).is_ok()
}

// ============================================================================
// Helpers
// ============================================================================

/// Double SHA-256 (hash of hash) commonly used in blockchains
#[wasm_bindgen]
pub fn crypto_msg_hash(data: &[u8]) -> String {
    let mut hasher1 = Sha256::new();
    hasher1.update(data);
    let hash1 = hasher1.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(hash1);
    let hash2 = hasher2.finalize();
    
    hex::encode(hash2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hello() {
        let hash = sha256(b"hello");
        assert_eq!(hash, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"key";
        let message = b"The quick brown fox jumps over the lazy dog";
        let hmac = hmac_sha256(key, message).unwrap();
        assert_eq!(hmac, "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8");
    }

    #[test]
    fn test_verify() {
        let key = b"secret_key";
        let message = b"trade:100kWh:4.5THB";
        let sig = hmac_sha256(key, message).unwrap();
        
        assert!(crypto_verify(key, message, &sig));
        assert!(!crypto_verify(key, b"wrong msg", &sig));
    }
}
