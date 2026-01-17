//! Order Book and Matching Engine
//! 
//! Client-side order book for visualization and matching preview.

use wasm_bindgen::prelude::*;
use std::cmp::Ordering;
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

// ============================================================================
// Order Book
// ============================================================================

#[wasm_bindgen]
pub struct OrderBook {
    bids: Vec<Order>,  // Sorted by price DESC, then timestamp ASC
    asks: Vec<Order>,  // Sorted by price ASC, then timestamp ASC
}

#[wasm_bindgen]
impl OrderBook {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            bids: Vec::with_capacity(1000),
            asks: Vec::with_capacity(1000),
        }
    }

    /// Clear all orders
    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
    }

    /// Add an order to the book
    pub fn add_order(&mut self, id: u32, side: u8, price: f64, quantity: f64, timestamp: u64) {
        let order = Order::new(id, Side::from(side), price, quantity, timestamp);
        match order.side {
            Side::Buy => {
                let pos = self.bids.binary_search_by(|probe| {
                    match probe.price.partial_cmp(&order.price).unwrap_or(Ordering::Equal) {
                        Ordering::Equal => probe.timestamp.cmp(&order.timestamp),
                        Ordering::Greater => Ordering::Less,
                        Ordering::Less => Ordering::Greater,
                    }
                }).unwrap_or_else(|pos| pos);
                self.bids.insert(pos, order);
            }
            Side::Sell => {
                let pos = self.asks.binary_search_by(|probe| {
                    match probe.price.partial_cmp(&order.price).unwrap_or(Ordering::Equal) {
                        Ordering::Equal => probe.timestamp.cmp(&order.timestamp),
                        other => other,
                    }
                }).unwrap_or_else(|pos| pos);
                self.asks.insert(pos, order);
            }
        }
    }

    /// Bulk check/add orders (not strictly necessary with wasm-bindgen if we just loop in JS, but nice for perf)
    pub fn load_orders(&mut self, orders: JsValue) -> Result<(), JsValue> {
        let orders_vec: Vec<Order> = serde_wasm_bindgen::from_value(orders)?;
        self.clear();
        for order in orders_vec {
             // Re-use logic to insert sorted
             // Ideally we'd just sort once at the end for bulk load, but this is safer
             // To avoid duplication, we call the internal adding logic or just replicate it.
             // For simplicity, let's just create a helper or call add_order logic.
             // But since we can't easily call `self.add_order` which takes flat params from here without verbosity...
             // Let's just trust the JS or re-implement insert logic.
             // Actually, for bulk load, just clearing and re-adding is fine.
             // We can optimize if needed.
             self.add_reused(order);
        }
        Ok(())
    }

    fn add_reused(&mut self, order: Order) {
         match order.side {
            Side::Buy => {
                let pos = self.bids.binary_search_by(|probe| {
                    match probe.price.partial_cmp(&order.price).unwrap_or(Ordering::Equal) {
                        Ordering::Equal => probe.timestamp.cmp(&order.timestamp),
                        Ordering::Greater => Ordering::Less,
                        Ordering::Less => Ordering::Greater,
                    }
                }).unwrap_or_else(|pos| pos);
                self.bids.insert(pos, order);
            }
            Side::Sell => {
                let pos = self.asks.binary_search_by(|probe| {
                    match probe.price.partial_cmp(&order.price).unwrap_or(Ordering::Equal) {
                        Ordering::Equal => probe.timestamp.cmp(&order.timestamp),
                        other => other,
                    }
                }).unwrap_or_else(|pos| pos);
                self.asks.insert(pos, order);
            }
        }
    }

    pub fn cancel_order(&mut self, order_id: u32) -> bool {
        if let Some(pos) = self.bids.iter().position(|o| o.id == order_id) {
            self.bids.remove(pos);
            return true;
        }
        if let Some(pos) = self.asks.iter().position(|o| o.id == order_id) {
            self.asks.remove(pos);
            return true;
        }
        false
    }

    pub fn best_bid_price(&self) -> f64 {
        self.bids.first().map(|o| o.price).unwrap_or(-1.0)
    }

    pub fn best_ask_price(&self) -> f64 {
        self.asks.first().map(|o| o.price).unwrap_or(-1.0)
    }

    pub fn spread(&self) -> f64 {
        match (self.bids.first(), self.asks.first()) {
            (Some(bid), Some(ask)) => ask.price - bid.price,
            _ => -1.0,
        }
    }

    pub fn mid_price(&self) -> f64 {
        match (self.bids.first(), self.asks.first()) {
             (Some(bid), Some(ask)) => (bid.price + ask.price) / 2.0,
             _ => -1.0,
        }
    }

    pub fn match_orders(&mut self) -> Result<JsValue, JsValue> {
        let mut matches = Vec::new();

        while !self.bids.is_empty() && !self.asks.is_empty() {
            let best_bid = &self.bids[0];
            let best_ask = &self.asks[0];

            if best_bid.price >= best_ask.price {
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

                if bid_remaining <= 0.0001 {
                    self.bids.remove(0);
                } else {
                    self.bids[0].quantity = bid_remaining;
                }

                if ask_remaining <= 0.0001 {
                    self.asks.remove(0);
                } else {
                    self.asks[0].quantity = ask_remaining;
                }
            } else {
                break;
            }
        }

        Ok(serde_wasm_bindgen::to_value(&matches)?)
    }

    /// Get depth data for visualization
    /// Returns: { bids: [[price, cum_qty], ...], asks: [[price, cum_qty], ...] }
    pub fn get_depth(&self, levels: usize) -> Result<JsValue, JsValue> {
        let mut bid_depth = Vec::with_capacity(levels);
        let mut cumulative = 0.0;
        let mut last_price = f64::NAN;

        for order in self.bids.iter().take(levels * 10) {
            if order.price != last_price {
                if !last_price.is_nan() && bid_depth.len() < levels {
                    bid_depth.push((last_price, cumulative));
                }
                last_price = order.price;
            }
            cumulative += order.quantity;
        }
        if !last_price.is_nan() && bid_depth.len() < levels {
            bid_depth.push((last_price, cumulative));
        }

        let mut ask_depth = Vec::with_capacity(levels);
        cumulative = 0.0;
        last_price = f64::NAN;

        for order in self.asks.iter().take(levels * 10) {
            if order.price != last_price {
                if !last_price.is_nan() && ask_depth.len() < levels {
                    ask_depth.push((last_price, cumulative));
                }
                last_price = order.price;
            }
            cumulative += order.quantity;
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
        self.bids.len()
    }

    pub fn ask_count(&self) -> usize {
        self.asks.len()
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
}
