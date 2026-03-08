//! Order Book and Matching Engine
//! 
//! Client-side order book for visualization and matching preview.
//! OPTIMIZED: Uses BTreeMap for O(log n) operations instead of Vec O(n)

use wasm_bindgen::prelude::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

impl From<u8> for Side {
    fn from(v: u8) -> Self {
        if v == 0 { Side::Buy } else { Side::Sell }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: u32,
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

impl Order {
    pub fn new(id: u32, side: Side, price: f64, quantity: f64, timestamp: u64) -> Self {
        Self { id, side, price, quantity, timestamp }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Match {
    pub buy_order_id: u32,
    pub sell_order_id: u32,
    pub price: f64,
    pub quantity: f64,
}

// Helper for reverse ordering in BTreeMap
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct ReversePrice(u64);

impl ReversePrice {
    fn from_f64(price: f64) -> Self {
        // Store as fixed-point to avoid floating point issues in ordering
        Self((price * 1_000_000.0) as u64)
    }
    
    fn to_f64(&self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }
}

// Helper for normal ordering in BTreeMap
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Price(u64);

impl Price {
    fn from_f64(price: f64) -> Self {
        Self((price * 1_000_000.0) as u64)
    }
    
    fn to_f64(&self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }
}

// ============================================================================
// Order Book - OPTIMIZED with BTreeMap
// ============================================================================

#[wasm_bindgen]
pub struct OrderBook {
    // Bids: Highest price first (Reverse ordering)
    // Key: Reverse price level, Value: Vec of orders at that price (time-ordered)
    bids: BTreeMap<ReversePrice, Vec<Order>>,
    // Asks: Lowest price first (Natural ordering)
    // Key: Price level, Value: Vec of orders at that price (time-ordered)
    asks: BTreeMap<Price, Vec<Order>>,
    // Index for O(1) order lookup by ID
    order_index: std::collections::HashMap<u32, (Side, u64)>, // (side, price_key)
}

#[wasm_bindgen]
impl OrderBook {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_index: std::collections::HashMap::with_capacity(1000),
        }
    }

    /// Clear all orders
    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.order_index.clear();
    }

    /// Add an order to the book - O(log n) insertion
    pub fn add_order(&mut self, id: u32, side: u8, price: f64, quantity: f64, timestamp: u64) {
        let order = Order::new(id, Side::from(side), price, quantity, timestamp);
        
        match order.side {
            Side::Buy => {
                let price_key = ReversePrice::from_f64(price);
                self.bids.entry(price_key)
                    .or_insert_with(Vec::new)
                    .push(order);
                self.order_index.insert(id, (Side::Buy, price_key.0));
            }
            Side::Sell => {
                let price_key = Price::from_f64(price);
                self.asks.entry(price_key)
                    .or_insert_with(Vec::new)
                    .push(order);
                self.order_index.insert(id, (Side::Sell, price_key.0));
            }
        }
    }

    /// Bulk load orders - optimized to avoid per-insert overhead
    pub fn load_orders(&mut self, orders: JsValue) -> Result<(), JsValue> {
        let orders_vec: Vec<Order> = serde_wasm_bindgen::from_value(orders)?;
        self.clear();
        
        // Group orders by price level for efficient batch insertion
        for order in orders_vec {
            match order.side {
                Side::Buy => {
                    let price_key = ReversePrice::from_f64(order.price);
                    self.bids.entry(price_key)
                        .or_insert_with(Vec::new)
                        .push(order);
                    self.order_index.insert(order.id, (Side::Buy, price_key.0));
                }
                Side::Sell => {
                    let price_key = Price::from_f64(order.price);
                    self.asks.entry(price_key)
                        .or_insert_with(Vec::new)
                        .push(order);
                    self.order_index.insert(order.id, (Side::Sell, price_key.0));
                }
            }
        }
        Ok(())
    }

    /// Cancel order - O(1) lookup with HashMap index
    pub fn cancel_order(&mut self, order_id: u32) -> bool {
        // O(1) lookup using index
        let Some((side, price_key)) = self.order_index.remove(&order_id) else {
            return false;
        };
        
        match side {
            Side::Buy => {
                let key = ReversePrice(price_key);
                if let Some(orders) = self.bids.get_mut(&key) {
                    if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                        orders.remove(pos);
                        // Clean up empty price levels
                        if orders.is_empty() {
                            self.bids.remove(&key);
                        }
                        return true;
                    }
                }
            }
            Side::Sell => {
                let key = Price(price_key);
                if let Some(orders) = self.asks.get_mut(&key) {
                    if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                        orders.remove(pos);
                        // Clean up empty price levels
                        if orders.is_empty() {
                            self.asks.remove(&key);
                        }
                        return true;
                    }
                }
            }
        }
        
        false
    }

    /// Get best bid price - O(1) with BTreeMap
    pub fn best_bid_price(&self) -> f64 {
        self.bids.first_key_value()
            .map(|(k, v)| if !v.is_empty() { k.to_f64() } else { -1.0 })
            .unwrap_or(-1.0)
    }

    /// Get best ask price - O(1) with BTreeMap
    pub fn best_ask_price(&self) -> f64 {
        self.asks.first_key_value()
            .map(|(k, v)| if !v.is_empty() { k.to_f64() } else { -1.0 })
            .unwrap_or(-1.0)
    }

    pub fn spread(&self) -> f64 {
        match (self.best_bid_price(), self.best_ask_price()) {
            (bid, ask) if bid >= 0.0 && ask >= 0.0 => ask - bid,
            _ => -1.0,
        }
    }

    pub fn mid_price(&self) -> f64 {
        match (self.best_bid_price(), self.best_ask_price()) {
            (bid, ask) if bid >= 0.0 && ask >= 0.0 => (bid + ask) / 2.0,
            _ => -1.0,
        }
    }

    /// Match orders - optimized with BTreeMap
    pub fn match_orders(&mut self) -> Result<JsValue, JsValue> {
        let mut matches = Vec::new();

        loop {
            // Get the best bid and ask by peeking at first entries
            let best_bid_opt = self.bids.first_key_value()
                .and_then(|(_, orders)| orders.first().map(|o| *o));
            let best_ask_opt = self.asks.first_key_value()
                .and_then(|(_, orders)| orders.first().map(|o| *o));

            let (best_bid, best_ask) = match (best_bid_opt, best_ask_opt) {
                (Some(bid), Some(ask)) => (bid, ask),
                _ => break, // No more orders to match
            };

            if best_bid.price < best_ask.price {
                break; // No more matches possible
            }

            let exec_price = if best_bid.timestamp <= best_ask.timestamp {
                best_bid.price
            } else {
                best_ask.price
            };

            let exec_qty = best_bid.quantity.min(best_ask.quantity);

            matches.push(Match {
                buy_order_id: best_bid.id,
                sell_order_id: best_ask.id,
                price: exec_price,
                quantity: exec_qty,
            });

            let bid_remaining = best_bid.quantity - exec_qty;
            let ask_remaining = best_ask.quantity - exec_qty;

            // Update or remove bid
            let bid_key = ReversePrice::from_f64(best_bid.price);
            if bid_remaining <= 0.0001 {
                if let Some(orders) = self.bids.get_mut(&bid_key) {
                    if !orders.is_empty() {
                        orders.remove(0);
                    }
                    self.order_index.remove(&best_bid.id);
                    if orders.is_empty() {
                        self.bids.remove(&bid_key);
                    }
                }
            } else {
                if let Some(orders) = self.bids.get_mut(&bid_key) {
                    if !orders.is_empty() {
                        orders[0].quantity = bid_remaining;
                    }
                }
            }

            // Update or remove ask
            let ask_key = Price::from_f64(best_ask.price);
            if ask_remaining <= 0.0001 {
                if let Some(orders) = self.asks.get_mut(&ask_key) {
                    if !orders.is_empty() {
                        orders.remove(0);
                    }
                    self.order_index.remove(&best_ask.id);
                    if orders.is_empty() {
                        self.asks.remove(&ask_key);
                    }
                }
            } else {
                if let Some(orders) = self.asks.get_mut(&ask_key) {
                    if !orders.is_empty() {
                        orders[0].quantity = ask_remaining;
                    }
                }
            }
        }

        Ok(serde_wasm_bindgen::to_value(&matches)?)
    }

    /// Get depth data for visualization - optimized iteration
    /// Returns: { bids: [[price, cum_qty], ...], asks: [[price, cum_qty], ...] }
    pub fn get_depth(&self, levels: usize) -> Result<JsValue, JsValue> {
        let mut bid_depth = Vec::with_capacity(levels);
        let mut cumulative = 0.0;
        let mut last_price = f64::NAN;
        let mut count = 0;

        // Iterate through price levels in order
        for (price_key, orders) in self.bids.iter() {
            if count >= levels {
                break;
            }
            
            let price = price_key.to_f64();
            if price != last_price {
                if !last_price.is_nan() {
                    bid_depth.push((last_price, cumulative));
                    count += 1;
                }
                last_price = price;
            }
            
            for order in orders.iter().take(levels * 10 - bid_depth.len() * 10) {
                cumulative += order.quantity;
            }
        }
        
        if !last_price.is_nan() && bid_depth.len() < levels {
            bid_depth.push((last_price, cumulative));
        }

        let mut ask_depth = Vec::with_capacity(levels);
        cumulative = 0.0;
        last_price = f64::NAN;
        count = 0;

        for (price_key, orders) in self.asks.iter() {
            if count >= levels {
                break;
            }
            
            let price = price_key.to_f64();
            if price != last_price {
                if !last_price.is_nan() {
                    ask_depth.push((last_price, cumulative));
                    count += 1;
                }
                last_price = price;
            }
            
            for order in orders.iter().take(levels * 10 - ask_depth.len() * 10) {
                cumulative += order.quantity;
            }
        }
        
        if !last_price.is_nan() && ask_depth.len() < levels {
            ask_depth.push((last_price, cumulative));
        }

        let result = DepthResult {
            bids: bid_depth,
            asks: ask_depth,
        };

        Ok(serde_wasm_bindgen::to_value(&result)?)
    }

    pub fn bid_count(&self) -> usize {
        self.bids.values().map(|v| v.len()).sum()
    }

    pub fn ask_count(&self) -> usize {
        self.asks.values().map(|v| v.len()).sum()
    }
}

#[derive(Serialize)]
struct DepthResult {
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_insertion() {
        let mut book = OrderBook::new();
        
        book.add_order(1, 0, 100.0, 10.0, 1);
        book.add_order(2, 0, 101.0, 5.0, 2);
        book.add_order(3, 1, 102.0, 8.0, 3);
        
        assert_eq!(book.best_bid_price(), 101.0);
        assert_eq!(book.best_ask_price(), 102.0);
    }

    #[test]
    fn test_cancel_order() {
        let mut book = OrderBook::new();
        
        book.add_order(1, 0, 100.0, 10.0, 1);
        book.add_order(2, 0, 101.0, 5.0, 2);
        
        assert!(book.cancel_order(1));
        assert!(!book.cancel_order(999));
        assert_eq!(book.bid_count(), 1);
    }
}
