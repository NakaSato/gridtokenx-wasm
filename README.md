# GridTokenX WASM Library

Optimized Rust library compiled to WebAssembly for high-performance client-side operations. This library handles heavy computational tasks such as energy grid topology analysis, map clustering, option pricing, and cryptographic signing.

## 🏗 Architecture

The library is improved to be thread-safe (safe for single-threaded WASM environments without race conditions) by avoiding `static mut` globals.

### Memory Model
Instead of unsafe mutable statics, we use `thread_local!` storage wrapped in `RefCell`. This ensures:
- **Safety**: No "shared reference to mutable static" undefined behavior.
- **Compliance**: Adheres to modern Rust strictness (Edition 2024 compatible).
- **Performance**: Zero-cost access in single-threaded WASM.

```rust
// Old (Unsafe)
static mut NETWORK: GridNetwork = ...;

// New (Safe)
thread_local! {
    static NETWORK: RefCell<GridNetwork> = RefCell::new(...);
}
```

## 📦 Modules

### 1. Topology (`topology.rs`)
Graph analysis for the energy distribution grid.
- **Features**: Shortest path finding (Dijkstra), power flow calculation, line loss estimation.
- **Key Exports**: `topology_shortest_path`, `topology_calc_flow`, `topology_calc_losses`.

### 2. Clustering (`clustering.rs`)
High-performance point clustering for map visualization.
- **Features**: Supercluster-like algorithm, Web Mercator projection.
- **Key Exports**: `get_clusters`.

### 3. Simulation (`simulation.rs`)
Time-based energy generation and consumption simulation.
- **Features**: Realistic fluctuation models, day/night cycles for solar/consumption.
- **Key Exports**: `update_simulation`.

### 4. Order Book (`orderbook.rs`)
Client-side matching engine for P2P market visualization.
- **Features**: Price-time priority matching, depth chart data generation.
- **Key Exports**: `orderbook_match`, `orderbook_depth`, `orderbook_add`.

### 5. Crypto (`crypto.rs`)
Zero-dependency cryptographic primitives for secure message signing.
- **Features**: SHA-256, HMAC-SHA256.
- **Key Exports**: `crypto_sign`, `crypto_verify`, `crypto_sha256`.

## 🛠 Building

Prerequisite: `wasm-pack` or `cargo build` with wasm target.

```bash
# Build optimized release binary
cargo build --target wasm32-unknown-unknown --release

# Resulting binary is located at:
# target/wasm32-unknown-unknown/release/gridtokenx_wasm.wasm
```
