//! Order Book and Matching Engine
//! 
//! Client-side order book for visualization and matching preview.
//! Orders are stored in sorted vectors for efficient best bid/ask access.

use std::cmp::Ordering;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

impl From<u8> for Side {
    fn from(v: u8) -> Self {
        if v == 0 { Side::Buy } else { Side::Sell }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Order {
    pub id: u32,
    pub side: Side,
    pub price: f64,      // Price per kWh
    pub quantity: f64,   // kWh
    pub timestamp: u64,  // For time priority
}

impl Order {
    pub fn new(id: u32, side: Side, price: f64, quantity: f64, timestamp: u64) -> Self {
        Self { id, side, price, quantity, timestamp }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Match {
    pub buy_order_id: u32,
    pub sell_order_id: u32,
    pub price: f64,
    pub quantity: f64,
}

// ============================================================================
// Order Book
// ============================================================================

/// Simple order book with sorted bids (descending) and asks (ascending)
pub struct OrderBook {
    bids: Vec<Order>,  // Sorted by price DESC, then timestamp ASC
    asks: Vec<Order>,  // Sorted by price ASC, then timestamp ASC
}

impl OrderBook {
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
    pub fn add_order(&mut self, order: Order) {
        match order.side {
            Side::Buy => {
                // Insert sorted: highest price first, then earliest timestamp
                let pos = self.bids.binary_search_by(|probe| {
                    match probe.price.partial_cmp(&order.price).unwrap_or(Ordering::Equal) {
                        Ordering::Equal => probe.timestamp.cmp(&order.timestamp),
                        Ordering::Greater => Ordering::Less,  // Higher price comes first
                        Ordering::Less => Ordering::Greater,
                    }
                }).unwrap_or_else(|pos| pos);
                self.bids.insert(pos, order);
            }
            Side::Sell => {
                // Insert sorted: lowest price first, then earliest timestamp
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

    /// Cancel an order by ID
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

    /// Get best bid (highest buy price)
    pub fn best_bid(&self) -> Option<&Order> {
        self.bids.first()
    }

    /// Get best ask (lowest sell price)
    pub fn best_ask(&self) -> Option<&Order> {
        self.asks.first()
    }

    /// Get spread (ask - bid)
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.price - bid.price),
            _ => None,
        }
    }

    /// Get mid price
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid.price + ask.price) / 2.0),
            _ => None,
        }
    }

    /// Match orders and return matches
    /// Uses price-time priority matching
    pub fn match_orders(&mut self) -> Vec<Match> {
        let mut matches = Vec::new();

        while !self.bids.is_empty() && !self.asks.is_empty() {
            let best_bid = &self.bids[0];
            let best_ask = &self.asks[0];

            // Check if prices cross (bid >= ask means a match)
            if best_bid.price >= best_ask.price {
                // Execute at the earlier order's price (maker price)
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

                // Update quantities
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
                // No more matches possible
                break;
            }
        }

        matches
    }

    /// Get depth data for visualization
    /// Returns: (bids: Vec<(price, cumulative_qty)>, asks: Vec<(price, cumulative_qty)>)
    pub fn get_depth(&self, levels: usize) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
        let mut bid_depth = Vec::with_capacity(levels);
        let mut ask_depth = Vec::with_capacity(levels);

        // Aggregate bids by price level
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

        // Aggregate asks by price level
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

        (bid_depth, ask_depth)
    }

    pub fn bid_count(&self) -> usize {
        self.bids.len()
    }

    pub fn ask_count(&self) -> usize {
        self.asks.len()
    }
}

// ============================================================================
// Global State & FFI
// ============================================================================

static mut ORDER_BOOK: Option<OrderBook> = None;
static mut MATCH_OUTPUT: Vec<f64> = Vec::new();
static mut DEPTH_OUTPUT: Vec<f64> = Vec::new();

fn get_book() -> &'static mut OrderBook {
    unsafe {
        if ORDER_BOOK.is_none() {
            ORDER_BOOK = Some(OrderBook::new());
        }
        ORDER_BOOK.as_mut().unwrap()
    }
}

/// Initialize/reset the order book
#[no_mangle]
pub extern "C" fn orderbook_init() {
    get_book().clear();
}

/// Add an order to the book
/// side: 0=Buy, 1=Sell
#[no_mangle]
pub extern "C" fn orderbook_add(id: u32, side: u8, price: f64, quantity: f64, timestamp: u64) {
    let order = Order::new(id, Side::from(side), price, quantity, timestamp);
    get_book().add_order(order);
}

/// Bulk load orders from buffer
/// Format: [id, side, price, quantity, timestamp, ...]
#[no_mangle]
pub extern "C" fn orderbook_load(ptr: *const f64, count: usize) {
    let input = unsafe { std::slice::from_raw_parts(ptr, count * 5) };
    let book = get_book();
    book.clear();

    for i in 0..count {
        let order = Order::new(
            input[i * 5] as u32,
            Side::from(input[i * 5 + 1] as u8),
            input[i * 5 + 2],
            input[i * 5 + 3],
            input[i * 5 + 4] as u64,
        );
        book.add_order(order);
    }
}

/// Cancel an order
#[no_mangle]
pub extern "C" fn orderbook_cancel(id: u32) -> u8 {
    if get_book().cancel_order(id) { 1 } else { 0 }
}

/// Get best bid price (returns -1 if no bids)
#[no_mangle]
pub extern "C" fn orderbook_best_bid() -> f64 {
    get_book().best_bid().map(|o| o.price).unwrap_or(-1.0)
}

/// Get best ask price (returns -1 if no asks)
#[no_mangle]
pub extern "C" fn orderbook_best_ask() -> f64 {
    get_book().best_ask().map(|o| o.price).unwrap_or(-1.0)
}

/// Get spread
#[no_mangle]
pub extern "C" fn orderbook_spread() -> f64 {
    get_book().spread().unwrap_or(-1.0)
}

/// Get mid price
#[no_mangle]
pub extern "C" fn orderbook_mid_price() -> f64 {
    get_book().mid_price().unwrap_or(-1.0)
}

/// Match orders and return count
/// Output format: [buy_id, sell_id, price, quantity, ...]
#[no_mangle]
pub extern "C" fn orderbook_match() -> usize {
    let matches = get_book().match_orders();
    unsafe {
        MATCH_OUTPUT.clear();
        for m in &matches {
            MATCH_OUTPUT.push(m.buy_order_id as f64);
            MATCH_OUTPUT.push(m.sell_order_id as f64);
            MATCH_OUTPUT.push(m.price);
            MATCH_OUTPUT.push(m.quantity);
        }
    }
    matches.len()
}

/// Get pointer to match output buffer
#[no_mangle]
pub extern "C" fn orderbook_match_ptr() -> *const f64 {
    unsafe { MATCH_OUTPUT.as_ptr() }
}

/// Get depth data for visualization
/// Output format: [bid_count, ask_count, bid_price, bid_qty, ..., ask_price, ask_qty, ...]
#[no_mangle]
pub extern "C" fn orderbook_depth(levels: usize) -> usize {
    let (bids, asks) = get_book().get_depth(levels);
    unsafe {
        DEPTH_OUTPUT.clear();
        DEPTH_OUTPUT.push(bids.len() as f64);
        DEPTH_OUTPUT.push(asks.len() as f64);
        for (p, q) in &bids {
            DEPTH_OUTPUT.push(*p);
            DEPTH_OUTPUT.push(*q);
        }
        for (p, q) in &asks {
            DEPTH_OUTPUT.push(*p);
            DEPTH_OUTPUT.push(*q);
        }
    }
    bids.len() + asks.len()
}

/// Get pointer to depth output buffer
#[no_mangle]
pub extern "C" fn orderbook_depth_ptr() -> *const f64 {
    unsafe { DEPTH_OUTPUT.as_ptr() }
}

/// Get order counts
#[no_mangle]
pub extern "C" fn orderbook_bid_count() -> usize {
    get_book().bid_count()
}

#[no_mangle]
pub extern "C" fn orderbook_ask_count() -> usize {
    get_book().ask_count()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_insertion() {
        let mut book = OrderBook::new();
        
        book.add_order(Order::new(1, Side::Buy, 100.0, 10.0, 1));
        book.add_order(Order::new(2, Side::Buy, 101.0, 5.0, 2));
        book.add_order(Order::new(3, Side::Sell, 102.0, 8.0, 3));
        
        assert_eq!(book.best_bid().unwrap().price, 101.0);
        assert_eq!(book.best_ask().unwrap().price, 102.0);
    }

    #[test]
    fn test_matching() {
        let mut book = OrderBook::new();
        
        book.add_order(Order::new(1, Side::Buy, 100.0, 10.0, 1));
        book.add_order(Order::new(2, Side::Sell, 99.0, 5.0, 2));  // Crosses!
        
        let matches = book.match_orders();
        
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].quantity, 5.0);
        assert_eq!(matches[0].price, 100.0);  // Buyer was first, so buyer's price
        
        // Remaining bid should be 5.0
        assert_eq!(book.bids[0].quantity, 5.0);
        assert!(book.asks.is_empty());
    }

    #[test]
    fn test_cancel() {
        let mut book = OrderBook::new();
        
        book.add_order(Order::new(1, Side::Buy, 100.0, 10.0, 1));
        assert_eq!(book.bid_count(), 1);
        
        assert!(book.cancel_order(1));
        assert_eq!(book.bid_count(), 0);
        
        assert!(!book.cancel_order(999));  // Non-existent
    }

    #[test]
    fn test_spread() {
        let mut book = OrderBook::new();
        
        book.add_order(Order::new(1, Side::Buy, 99.0, 10.0, 1));
        book.add_order(Order::new(2, Side::Sell, 101.0, 10.0, 2));
        
        assert_eq!(book.spread().unwrap(), 2.0);
        assert_eq!(book.mid_price().unwrap(), 100.0);
    }
}
