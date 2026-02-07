use solana_zk_token_sdk::encryption::{
    elgamal::ElGamalKeypair,
    pedersen::Pedersen,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

fn main() {
    // 1. Create a dummy keypair
    let keypair = ElGamalKeypair::new_rand();
    let pubkey = keypair.pubkey();
    
    // 2. Encrypt some values
    let price = 8_000_000_000_000_000u64;
    let amount = 46_279u64;
    
    let (_, price_opening) = Pedersen::new(price);
    let price_ciphertext = pubkey.encrypt_with(price, &price_opening);
    
    let (_, amount_opening) = Pedersen::new(amount);
    let amount_ciphertext = pubkey.encrypt_with(amount, &amount_opening);
    
    // 3. Convert to bytes (64 bytes each)
    let price_bytes: [u8; 64] = unsafe { std::mem::transmute_copy(&price_ciphertext) };
    let amount_bytes: [u8; 64] = unsafe { std::mem::transmute_copy(&amount_ciphertext) };
    
    println!("Price Ciphertext (Base64): {}", BASE64.encode(&price_bytes));
    println!("Amount Ciphertext (Base64): {}", BASE64.encode(&amount_bytes));
}
