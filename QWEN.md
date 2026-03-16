# GridTokenX WASM - Project Context

## Project Overview

**GridTokenX WASM** is a high-performance Rust library compiled to WebAssembly (WASM) for the GridTokenX decentralized energy trading platform. It handles computationally intensive client-side operations including:

- Energy auction simulation and market clearing
- Zero-knowledge proofs (ElGamal, Pedersen commitments, range proofs)
- Cryptographic operations (SHA-256, HMAC-SHA256)
- P2P order book matching engine
- Energy flow visualization (Bezier curves)
- Solana governance client with ZK-weighted voting
- Time-based energy generation/consumption simulation

**Version**: 0.1.1 | **Edition**: Rust 2021 | **Last Updated**: February 2026

---

## Architecture

### Module Structure

The library uses a modular architecture with `#[wasm_bindgen]` struct-based instances for each domain:

```
src/
├── lib.rs              # Main library entry point, re-exports all modules
└── modules/
    ├── mod.rs          # Module declarations
    ├── aggregation.rs  # Energy data aggregation
    ├── auction.rs      # Uniform clearing price auction (MCP calculation)
    ├── bezier.rs       # Quadratic Bezier curves for energy flow visualization
    ├── clustering.rs   # Energy profile archetype clustering
    ├── crypto.rs       # SHA-256, HMAC-SHA256, message signing
    ├── governance.rs   # Solana governance client with ZK voting
    ├── orderbook.rs    # Order matching engine with depth chart
    ├── portfolio.rs    # Aggregated portfolio risk analytics
    ├── pricing.rs      # Black-Scholes and Greeks calculations
    ├── simulation.rs   # Energy node and flow simulation
    └── zk.rs           # Zero-knowledge proofs (ElGamal, Pedersen)
```

### Key Design Patterns

1. **Struct-based WASM exports**: Each module exposes a class that can be instantiated independently, providing safe, isolated state management in the single-threaded WASM environment.

2. **BTreeMap optimization**: Performance-critical modules (e.g., `orderbook.rs`) use `BTreeMap` for O(log n) operations instead of Vec O(n).

3. **Serialization**: All data structures use `serde` with `serde-wasm-bindgen` for seamless Rust ↔ JavaScript data transfer.

4. **Panic handling**: Uses `console_error_panic_hook` for debugging WASM panics in the browser.

---

## Building and Running

### Prerequisites

```bash
# Install wasm-pack
cargo install wasm-pack

# Ensure Rust toolchain has WASM target
rustup target add wasm32-unknown-unknown
```

### Build Commands

```bash
# Build for bundler (webpack/vite) - outputs to pkg/
wasm-pack build --target bundler --out-dir pkg

# Build for Node.js - outputs to pkg-node/
wasm-pack build --target nodejs --out-dir pkg-node

# Build for web (ESM) - outputs to pkg-web/
wasm-pack build --target web --out-dir pkg-web

# Build raw WASM binary (no JS bindings)
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Build Output Targets

| Target | Directory | Use Case |
|--------|-----------|----------|
| Bundler | `pkg/` | Bundled web applications (webpack, vite) |
| Node.js | `pkg-node/` | Server-side Node.js usage |
| Web | `pkg-web/` | Direct browser `<script type="module">` |

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `wasm-bindgen` | 0.2 | Rust ↔ JavaScript interop |
| `js-sys` | 0.3 | JavaScript standard library bindings |
| `serde` | 1.0 (+derive) | Serialization/deserialization |
| `serde-wasm-bindgen` | 0.6 | Serde ↔ WASM value conversion |
| `sha2` | 0.10 | SHA-256 hashing |
| `hmac` | 0.12 | HMAC-SHA256 signing |
| `hex` | 0.4 | Hex encoding/decoding |
| `solana-zk-token-sdk` | 2.3.1 | ElGamal, Pedersen commitments, range proofs |
| `console_error_panic_hook` | 0.1 | WASM panic debugging |
| `getrandom` | 0.2 (+js) | Random number generation in WASM |
| `curve25519-dalek` | 3 (+serde) | Elliptic curve cryptography |
| `rand` | 0.8 | Random number generation |
| `bytemuck` | 1.16 (+derive) | Zero-copy memory operations |

---

## Module Exports Summary

### Simulation (`simulation.rs`)
Time-based energy generation and consumption simulation with realistic fluctuation models.

| Export | Description |
|--------|-------------|
| `Simulation::new()` | Create simulation with seeded RNG |
| `Simulation::set_nodes()` / `set_flows()` | Load simulation data |
| `Simulation::update(hour, minute)` | Advance simulation tick |
| `Simulation::get_nodes()` / `get_flows()` | Get current state |
| `Simulation::get_grid_totals()` | Calculate aggregate totals |

### Order Book (`orderbook.rs`)
Client-side matching engine for P2P market visualization with price-time priority matching.

| Export | Description |
|--------|-------------|
| `OrderBook::new()` | Create empty order book |
| `OrderBook::add_order()` | Add order (0=Buy, 1=Sell) |
| `OrderBook::cancel_order()` | Cancel by ID |
| `OrderBook::match_orders()` | Execute matching |
| `OrderBook::get_depth()` | Depth chart data |
| `OrderBook::best_bid_price()` / `best_ask_price()` | Best prices |
| `OrderBook::spread()` / `mid_price()` | Spread calculations |

### Crypto (`crypto.rs`)
Cryptographic primitives using `sha2` and `hmac` crates.

| Export | Description |
|--------|-------------|
| `sha256(data)` | SHA-256 hash (hex) |
| `hmac_sha256(key, message)` | HMAC signature (hex) |
| `crypto_verify(key, message, sig)` | Verify HMAC signature |
| `crypto_msg_hash(data)` | Double SHA-256 |
| `sign_p2p_order()` | P2P order message signing |

### Bezier (`bezier.rs`)
Quadratic Bezier curve generation for energy flow visualization.

| Export | Description |
|--------|-------------|
| `calculate_bezier()` | Generate curve points as `[x, y, ...]` buffer |

### Auction (`auction.rs`)
Uniform clearing price auction simulator (Market Clearing Price).

| Export | Description |
|--------|-------------|
| `AuctionSimulator::new()` | Create empty auction |
| `AuctionSimulator::add_order()` | Add bid/ask |
| `AuctionSimulator::clear()` | Remove all orders |
| `AuctionSimulator::calculate_clearing_price()` | Compute MCP |

### ZK - Zero-Knowledge Proofs (`zk.rs`)
ElGamal keypairs, Pedersen commitments, and range proofs using `solana-zk-token-sdk`.

| Export | Description |
|--------|-------------|
| `WasmElGamalKeypair::new()` | Generate random keypair |
| `WasmElGamalKeypair::fromSecret()` | Recover from secret bytes |
| `WasmElGamalKeypair::pubkey()` / `secret()` | Get keys |
| `create_commitment()` | Pedersen commitment |
| `create_range_proof()` | Range proof for u64 |
| `create_transfer_proof()` | Full transfer proof |
| `derive_stealth_key()` | Stealth key derivation |

### Governance (`governance.rs`)
Solana governance client with ZK-weighted voting.

| Export | Description |
|--------|-------------|
| `GovernanceClient::new()` | Create client for Solana |
| `GovernanceClient::connect()` | Initialize connection |
| `GovernanceClient::vote_private()` | Cast ZK-weighted vote |
| `GovernanceClient::create_proposal()` | Create proposal |
| `generate_zk_vote_proof()` / `verify_zk_vote_proof()` | ZK proof operations |

### Global
| Export | Description |
|--------|-------------|
| `init_panic_hook()` | Initialize WASM panic handler |

---

## Development Conventions

### Code Style

1. **Module documentation**: Each module starts with a doc comment describing its purpose.
2. **WASM exports**: All public functions use `#[wasm_bindgen]` attribute.
3. **Error handling**: Use `Result<T, JsValue>` for functions that can fail in JavaScript context.
4. **Serialization**: All data structures derive `Serialize` and `Deserialize` for JS interop.
5. **Naming**: Rust snake_case for functions, JavaScript camelCase via `#[wasm_bindgen(js_name = "...")]` when needed.

### Testing Practices

- Unit tests are included in each module using `#[cfg(test)]` modules.
- Tests use standard `assert_eq!` and `assert!` macros.
- Run tests with `cargo test` (native Rust tests, not WASM).

### Performance Considerations

1. **BTreeMap over Vec**: Used in `orderbook.rs` for O(log n) lookups.
2. **HashMap index**: Order cancellation uses O(1) lookup via `order_index`.
3. **Seeded RNG**: Simulation module uses deterministic LCG for reproducibility.
4. **Fixed-point pricing**: Order book stores prices as fixed-point integers to avoid floating-point comparison issues.

---

## Usage from JavaScript

```javascript
import init, {
  init_panic_hook,
  Simulation,
  OrderBook,
  sha256,
  WasmElGamalKeypair,
} from "./pkg/gridtokenx_wasm.js";

await init();
init_panic_hook();

// Example: Create simulation
const sim = new Simulation();
sim.set_nodes(nodesJson);
sim.update(14, 30); // 2:30 PM
const currentState = sim.get_nodes();

// Example: Order book
const book = new OrderBook();
book.add_order(1, 0, 100.0, 10.0, Date.now()); // Buy order
book.add_order(2, 1, 102.0, 8.0, Date.now());  // Sell order
const matches = book.match_orders();
```

---

## Related Files

- `Cargo.toml` - Package configuration and dependencies
- `README.md` - User-facing documentation with detailed API tables
- `.gitignore` - Excludes build artifacts (`/pkg`, `/target`, etc.)
