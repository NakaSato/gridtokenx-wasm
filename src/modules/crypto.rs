//! Cryptographic Operations Module
//! 
//! Signing, verification, and hashing for P2P trade messages.
//! Uses a simple implementation without external crates to minimize bundle size.

// ============================================================================
// SHA-256 Implementation (simplified)
// ============================================================================

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const H_INIT: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

fn rotr(x: u32, n: u32) -> u32 {
    (x >> n) | (x << (32 - n))
}

fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

fn sigma0(x: u32) -> u32 {
    rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22)
}

fn sigma1(x: u32) -> u32 {
    rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25)
}

fn gamma0(x: u32) -> u32 {
    rotr(x, 7) ^ rotr(x, 18) ^ (x >> 3)
}

fn gamma1(x: u32) -> u32 {
    rotr(x, 17) ^ rotr(x, 19) ^ (x >> 10)
}

/// Compute SHA-256 hash of input bytes
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = H_INIT;

    // Pre-processing: adding padding bits
    let ml = data.len() as u64 * 8; // Message length in bits
    let mut padded = data.to_vec();
    padded.push(0x80);

    // Pad to 448 mod 512 bits (56 mod 64 bytes)
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }

    // Append original length as 64-bit big-endian
    padded.extend_from_slice(&ml.to_be_bytes());

    // Process each 512-bit (64-byte) chunk
    for chunk in padded.chunks(64) {
        let mut w = [0u32; 64];

        // Copy chunk into first 16 words
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }

        // Extend the first 16 words into the remaining 48 words
        for i in 16..64 {
            w[i] = gamma1(w[i - 2])
                .wrapping_add(w[i - 7])
                .wrapping_add(gamma0(w[i - 15]))
                .wrapping_add(w[i - 16]);
        }

        // Initialize working variables
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        // Compression loop
        for i in 0..64 {
            let t1 = hh
                .wrapping_add(sigma1(e))
                .wrapping_add(ch(e, f, g))
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let t2 = sigma0(a).wrapping_add(maj(a, b, c));

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        // Add compressed chunk to current hash value
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    // Produce the final hash value (big-endian)
    let mut result = [0u8; 32];
    for i in 0..8 {
        result[i * 4..i * 4 + 4].copy_from_slice(&h[i].to_be_bytes());
    }
    result
}

// ============================================================================
// Simple HMAC-SHA256
// ============================================================================

/// Compute HMAC-SHA256
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let block_size = 64;

    // If key is longer than block size, hash it
    let key_hash: [u8; 32];
    let key_prime: &[u8] = if key.len() > block_size {
        key_hash = sha256(key);
        &key_hash
    } else {
        key
    };

    // Pad key to block size
    let mut k_padded = [0u8; 64];
    k_padded[..key_prime.len()].copy_from_slice(key_prime);

    // Create inner and outer padding
    let mut o_key_pad = [0u8; 64];
    let mut i_key_pad = [0u8; 64];
    for i in 0..64 {
        o_key_pad[i] = k_padded[i] ^ 0x5c;
        i_key_pad[i] = k_padded[i] ^ 0x36;
    }

    // Inner hash
    let mut inner_data = i_key_pad.to_vec();
    inner_data.extend_from_slice(message);
    let inner_hash = sha256(&inner_data);

    // Outer hash
    let mut outer_data = o_key_pad.to_vec();
    outer_data.extend_from_slice(&inner_hash);
    sha256(&outer_data)
}

// ============================================================================
// Hex encoding utilities
// ============================================================================

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

/// Convert bytes to hex string (stored in static buffer)
fn bytes_to_hex(bytes: &[u8], output: &mut [u8]) {
    for (i, byte) in bytes.iter().enumerate() {
        output[i * 2] = HEX_CHARS[(byte >> 4) as usize];
        output[i * 2 + 1] = HEX_CHARS[(byte & 0x0f) as usize];
    }
}

fn hex_to_byte(hex: u8) -> Option<u8> {
    match hex {
        b'0'..=b'9' => Some(hex - b'0'),
        b'a'..=b'f' => Some(hex - b'a' + 10),
        b'A'..=b'F' => Some(hex - b'A' + 10),
        _ => None,
    }
}

/// Convert hex string to bytes
fn hex_to_bytes(hex: &[u8], output: &mut [u8]) -> bool {
    if hex.len() % 2 != 0 {
        return false;
    }
    for i in 0..output.len() {
        let high = match hex_to_byte(hex[i * 2]) {
            Some(v) => v,
            None => return false,
        };
        let low = match hex_to_byte(hex[i * 2 + 1]) {
            Some(v) => v,
            None => return false,
        };
        output[i] = (high << 4) | low;
    }
    true
}

// ============================================================================
// Global State & FFI
// ============================================================================

static mut HASH_OUTPUT: [u8; 32] = [0u8; 32];
static mut HEX_OUTPUT: [u8; 64] = [0u8; 64];
static mut SIG_OUTPUT: [u8; 64] = [0u8; 64];
static mut VERIFY_RESULT: u8 = 0;

/// Hash a message using SHA-256
/// Input: pointer to message bytes, length
/// Returns: 32 (hash length)
#[no_mangle]
pub extern "C" fn crypto_sha256(ptr: *const u8, len: usize) -> usize {
    let message = unsafe { std::slice::from_raw_parts(ptr, len) };
    let hash = sha256(message);
    unsafe {
        HASH_OUTPUT = hash;
    }
    32
}

/// Get pointer to hash output (32 bytes)
#[no_mangle]
pub extern "C" fn crypto_hash_ptr() -> *const u8 {
    unsafe { HASH_OUTPUT.as_ptr() }
}

/// Get hash as hex string
/// Returns: 64 (hex string length)
#[no_mangle]
pub extern "C" fn crypto_hash_hex() -> usize {
    unsafe {
        bytes_to_hex(&HASH_OUTPUT, &mut HEX_OUTPUT);
    }
    64
}

/// Get pointer to hex output (64 bytes)
#[no_mangle]
pub extern "C" fn crypto_hex_ptr() -> *const u8 {
    unsafe { HEX_OUTPUT.as_ptr() }
}

/// Sign a message using HMAC-SHA256
/// key_ptr: pointer to key bytes
/// key_len: key length
/// msg_ptr: pointer to message bytes
/// msg_len: message length
/// Returns: 32 (signature length)
#[no_mangle]
pub extern "C" fn crypto_sign(
    key_ptr: *const u8, key_len: usize,
    msg_ptr: *const u8, msg_len: usize
) -> usize {
    let key = unsafe { std::slice::from_raw_parts(key_ptr, key_len) };
    let message = unsafe { std::slice::from_raw_parts(msg_ptr, msg_len) };
    let sig = hmac_sha256(key, message);
    unsafe {
        SIG_OUTPUT[..32].copy_from_slice(&sig);
    }
    32
}

/// Get pointer to signature output (32 bytes for HMAC-SHA256)
#[no_mangle]
pub extern "C" fn crypto_sig_ptr() -> *const u8 {
    unsafe { SIG_OUTPUT.as_ptr() }
}

/// Verify an HMAC-SHA256 signature
/// key_ptr: pointer to key bytes
/// key_len: key length
/// msg_ptr: pointer to message bytes
/// msg_len: message length
/// sig_ptr: pointer to expected signature (32 bytes)
/// Returns: 1 if valid, 0 if invalid
#[no_mangle]
pub extern "C" fn crypto_verify(
    key_ptr: *const u8, key_len: usize,
    msg_ptr: *const u8, msg_len: usize,
    sig_ptr: *const u8
) -> u8 {
    let key = unsafe { std::slice::from_raw_parts(key_ptr, key_len) };
    let message = unsafe { std::slice::from_raw_parts(msg_ptr, msg_len) };
    let expected_sig = unsafe { std::slice::from_raw_parts(sig_ptr, 32) };
    
    let computed_sig = hmac_sha256(key, message);
    
    // Constant-time comparison to prevent timing attacks
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= computed_sig[i] ^ expected_sig[i];
    }
    
    if diff == 0 { 1 } else { 0 }
}

/// Create a message hash for signing (double SHA-256, like Bitcoin)
#[no_mangle]
pub extern "C" fn crypto_msg_hash(ptr: *const u8, len: usize) -> usize {
    let message = unsafe { std::slice::from_raw_parts(ptr, len) };
    let hash1 = sha256(message);
    let hash2 = sha256(&hash1);
    unsafe {
        HASH_OUTPUT = hash2;
    }
    32
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let hash = sha256(b"");
        // SHA-256 of empty string
        let expected = [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
            0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
            0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
            0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_hello() {
        let hash = sha256(b"hello");
        // SHA-256 of "hello"
        let expected = [
            0x2c, 0xf2, 0x4d, 0xba, 0x5f, 0xb0, 0xa3, 0x0e,
            0x26, 0xe8, 0x3b, 0x2a, 0xc5, 0xb9, 0xe2, 0x9e,
            0x1b, 0x16, 0x1e, 0x5c, 0x1f, 0xa7, 0x42, 0x5e,
            0x73, 0x04, 0x33, 0x62, 0x93, 0x8b, 0x98, 0x24,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"key";
        let message = b"The quick brown fox jumps over the lazy dog";
        let hmac = hmac_sha256(key, message);
        
        // Known HMAC-SHA256 result for this key/message pair
        let expected = [
            0xf7, 0xbc, 0x83, 0xf4, 0x30, 0x53, 0x84, 0x24,
            0xb1, 0x32, 0x98, 0xe6, 0xaa, 0x6f, 0xb1, 0x43,
            0xef, 0x4d, 0x59, 0xa1, 0x49, 0x46, 0x17, 0x59,
            0x97, 0x47, 0x9d, 0xbc, 0x2d, 0x1a, 0x3c, 0xd8,
        ];
        assert_eq!(hmac, expected);
    }

    #[test]
    fn test_verify() {
        let key = b"secret_key";
        let message = b"trade:100kWh:4.5THB";
        let sig = hmac_sha256(key, message);
        
        // Verify correct signature
        let mut diff = 0u8;
        let computed = hmac_sha256(key, message);
        for i in 0..32 {
            diff |= computed[i] ^ sig[i];
        }
        assert_eq!(diff, 0);
        
        // Verify wrong message fails
        let wrong_msg = b"trade:200kWh:4.5THB";
        let computed = hmac_sha256(key, wrong_msg);
        let mut diff = 0u8;
        for i in 0..32 {
            diff |= computed[i] ^ sig[i];
        }
        assert_ne!(diff, 0);
    }

    #[test]
    fn test_hex_encoding() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        let mut hex = [0u8; 8];
        bytes_to_hex(&bytes, &mut hex);
        assert_eq!(&hex, b"deadbeef");
    }
}
