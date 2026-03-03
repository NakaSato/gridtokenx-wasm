//! GridTokenX WASM Library
//! 
//! High-performance Rust/WASM modules for the GridTokenX trading platform.
//! 
//! ## Modules
//!
//! - **aggregation**: High-performance energy data aggregation
//! - **auction**: Auction mechanism for energy trading
//! - **bezier**: Quadratic Bezier curves for energy flow visualization
//! - **clustering**: Energy profile archetype clustering
//! - **crypto**: SHA-256 and HMAC-SHA256 cryptographic operations
//! - **governance**: Solana governance client with ZK-weighted voting
//! - **orderbook**: Order matching engine with depth chart
//! - **portfolio**: Aggregated portfolio risk analytics
//! - **pricing**: Black-Scholes and Greeks calculations
//! - **simulation**: Energy node and flow simulation
//! - **zk**: Zero-knowledge proofs (ElGamal, Pedersen)

mod modules;

pub use modules::aggregation::*;
pub use modules::auction::*;
pub use modules::bezier::*;
pub use modules::clustering::*;
pub use modules::crypto::*;
pub use modules::governance::*;
pub use modules::orderbook::*;
pub use modules::portfolio::*;
pub use modules::pricing::*;
pub use modules::simulation::*;
pub use modules::zk::*;

use wasm_bindgen::prelude::*;

/// Initializes the panic hook for the WASM library.
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
