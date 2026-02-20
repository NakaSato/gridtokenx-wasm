# GridTokenX WASM Library

> **Version**: 0.1.1  
> **Last Updated**: February 2026

Optimized Rust library compiled to WebAssembly for high-performance client-side operations. This library handles heavy computational tasks such as energy auction simulation, zero-knowledge proofs, cryptographic operations, and P2P trading.

## 🏗 Architecture

The library uses `#[wasm_bindgen]` struct-based instances for each module, providing safe, isolated state management in the single-threaded WASM environment. Each module exposes a class that can be instantiated independently.

### Dependencies

| Crate | Purpose |
|-------|---------|
| `wasm-bindgen` | Rust ↔ JS interop |
| `serde` / `serde-wasm-bindgen` | Serialization |
| `sha2` | SHA-256 hashing |
| `hmac` | HMAC-SHA256 signing |
| `hex` | Hex encoding |
| `solana-zk-token-sdk` | ElGamal, Pedersen commitments, range proofs |
| `console_error_panic_hook` | WASM panic debugging |

## 📦 Modules

### 1. Simulation (`simulation.rs`)
Time-based energy generation and consumption simulation.
- **Features**: Realistic fluctuation models, day/night cycles for solar/consumption, random status changes.

| Export | Signature | Description |
|--------|-----------|-------------|
| `Simulation::new()` | `() → Simulation` | Create simulation with seeded RNG |
| `Simulation::set_nodes()` | `(nodes: JsValue) → Result` | Load simulation nodes |
| `Simulation::set_flows()` | `(flows: JsValue) → Result` | Load simulation flows |
| `Simulation::update()` | `(hour: f64, minute: f64)` | Advance simulation tick with time-of-day multipliers |
| `Simulation::get_nodes()` | `() → JsValue` | Get current node states |
| `Simulation::get_flows()` | `() → JsValue` | Get current flow states |

### 2. Order Book (`orderbook.rs`)
Client-side matching engine for P2P market visualization.
- **Features**: Price-time priority matching, depth chart data, spread/mid-price calculations.

| Export | Signature | Description |
|--------|-----------|-------------|
| `OrderBook::new()` | `() → OrderBook` | Create empty order book (capacity 1000/side) |
| `OrderBook::add_order()` | `(id, side, price, quantity, timestamp)` | Add order (0=Buy, 1=Sell) |
| `OrderBook::load_orders()` | `(orders: JsValue) → Result` | Bulk load orders |
| `OrderBook::cancel_order()` | `(order_id: u32) → bool` | Cancel order by ID |
| `OrderBook::match_orders()` | `() → Vec<Match>` | Execute price-time priority matching |
| `OrderBook::get_depth()` | `(levels: usize) → DepthData` | Depth chart: `{ bids, asks }` with cumulative quantities |
| `OrderBook::best_bid_price()` | `() → f64` | Highest bid (-1.0 if empty) |
| `OrderBook::best_ask_price()` | `() → f64` | Lowest ask (-1.0 if empty) |
| `OrderBook::spread()` | `() → f64` | Ask − Bid spread |
| `OrderBook::mid_price()` | `() → f64` | (Bid + Ask) / 2 |
| `OrderBook::bid_count()` | `() → usize` | Number of bid orders |
| `OrderBook::ask_count()` | `() → usize` | Number of ask orders |
| `OrderBook::clear()` | `()` | Clear all orders |

### 3. Crypto (`crypto.rs`)
Cryptographic primitives for secure message signing and verification using `sha2` and `hmac` crates.
- **Features**: SHA-256, double SHA-256, HMAC-SHA256 signing and verification.

| Export | Signature | Description |
|--------|-----------|-------------|
| `sha256()` | `(data: &[u8]) → String` | SHA-256 hash (hex-encoded) |
| `hmac_sha256()` | `(key: &[u8], message: &[u8]) → String` | HMAC-SHA256 signature (hex-encoded) |
| `crypto_verify()` | `(key: &[u8], message: &[u8], signature_hex: &str) → bool` | Verify HMAC-SHA256 signature |
| `crypto_msg_hash()` | `(data: &[u8]) → String` | Double SHA-256 (blockchain-style) |

### 4. Bezier (`bezier.rs`)
Quadratic Bezier curve generation for energy flow visualization on maps.

| Export | Signature | Description |
|--------|-----------|-------------|
| `calculate_bezier()` | `(x1, y1, x2, y2, curve_intensity, segments) → Float64Array` | Generate Bezier curve points as flat `[x, y, x, y, ...]` buffer |

### 5. Auction (`auction.rs`)
Uniform clearing price auction simulator for energy markets (Market Clearing Price calculation).

| Export | Signature | Description |
|--------|-----------|-------------|
| `AuctionSimulator::new()` | `() → AuctionSimulator` | Create empty auction |
| `AuctionSimulator::add_order()` | `(id, price, amount, is_bid)` | Add a bid or ask order |
| `AuctionSimulator::clear()` | `()` | Remove all orders |
| `AuctionSimulator::calculate_clearing_price()` | `() → [clearing_price, clearing_volume]` | Compute uniform Market Clearing Price |

### 6. ZK — Zero-Knowledge Proofs (`zk.rs`)
ElGamal keypairs, Pedersen commitments, and range proofs for confidential energy trading. Built on `solana-zk-token-sdk`.

| Export | Signature | Description |
|--------|-----------|-------------|
| `WasmElGamalKeypair::new()` | `() → WasmElGamalKeypair` | Generate random ElGamal keypair |
| `WasmElGamalKeypair::fromSecret()` | `(secret: &[u8]) → Result` | Recover keypair from secret bytes |
| `WasmElGamalKeypair::pubkey()` | `() → Vec<u8>` | Get 32-byte public key |
| `WasmElGamalKeypair::secret()` | `() → Vec<u8>` | Get 32-byte secret key |
| `create_commitment()` | `(value: u64, blinding: &[u8]) → WasmCommitment` | Pedersen commitment with 32-byte blinding factor |
| `create_range_proof()` | `(amount: u64, blinding: &[u8]) → WasmRangeProof` | Range proof for u64 amount |
| `create_transfer_proof()` | `(amount, sender_balance, sender_blinding, amount_blinding) → WasmTransferProof` | Full transfer proof with balance equality |

### Global

| Export | Signature | Description |
|--------|-----------|-------------|
| `init_panic_hook()` | `()` | Initialize WASM panic handler for debugging |

## 📦 Build Targets

The library is compiled to three WASM targets:

| Target | Directory | Use Case |
|--------|-----------|----------|
| `pkg/` | Bundler (webpack/vite) | For bundled web applications |
| `pkg-node/` | Node.js | Server-side usage |
| `pkg-web/` | Web (ESM) | Direct browser `<script type="module">` |

> **Note**: Ensure all targets are rebuilt after source changes. Run `wasm-pack build` for each target.

## 🛠 Building

Prerequisite: `wasm-pack` or `cargo build` with wasm target.

```bash
# Build for bundler (default)
wasm-pack build --target bundler --out-dir pkg

# Build for Node.js
wasm-pack build --target nodejs --out-dir pkg-node

# Build for web (ESM)
wasm-pack build --target web --out-dir pkg-web

# Or build raw WASM binary
cargo build --target wasm32-unknown-unknown --release
```
