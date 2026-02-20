//! GridTokenX WASM Library
//! 
//! High-performance Rust/WASM modules for the GridTokenX trading platform.
//! 
//! ## Modules
//!
//! - **bezier**: Quadratic Bezier curves for energy flow visualization
//! - **crypto**: SHA-256 and HMAC-SHA256 cryptographic operations
//! - **orderbook**: Order matching engine with depth chart
//! - **simulation**: Energy node and flow simulation
//! - **zk**: Zero-knowledge proofs (ElGamal, Pedersen)

mod modules;

pub use modules::auction::*;
pub use modules::bezier::*;
pub use modules::crypto::*;
pub use modules::orderbook::*;
pub use modules::simulation::*;
pub use modules::zk::*;

use wasm_bindgen::prelude::*;

/// Initializes the panic hook for the WASM library.
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
