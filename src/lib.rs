//! GridTokenX WASM Library
//! 
//! High-performance Rust/WASM modules for the GridTokenX trading platform.
//! 
//! ## Modules
//! 
//! - **bezier**: Quadratic Bezier curves for energy flow visualization
//! - **clustering**: Map marker clustering (legacy, still present)
//! - **crypto**: SHA-256 and HMAC-SHA256 cryptographic operations
//! - **options**: Black-Scholes pricing (legacy, still present)
//! - **orderbook**: Order matching engine with depth chart
//! - **simulation**: Energy node and flow simulation
//! - **topology**: Grid network path finding (legacy, still present)

mod modules;

// Re-export all FFI functions from modules
pub use modules::bezier::*;
pub use modules::clustering::*;
pub use modules::crypto::*;
pub use modules::options::*;
pub use modules::orderbook::*;
pub use modules::simulation::*;
pub use modules::topology::*;

use wasm_bindgen::prelude::*;



#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
