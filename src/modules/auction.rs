use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct AuctionOrderWasm {
    pub id: u32,
    pub price: f64,
    pub amount: f64,
    pub is_bid: bool,
}

#[wasm_bindgen]
pub struct AuctionSimulator {
    orders: Vec<AuctionOrderWasm>,
}

#[wasm_bindgen]
impl AuctionSimulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { orders: Vec::new() }
    }

    pub fn add_order(&mut self, id: u32, price: f64, amount: f64, is_bid: bool) {
        self.orders.push(AuctionOrderWasm { id, price, amount, is_bid });
    }

    pub fn clear(&mut self) {
        self.orders.clear();
    }

    /// Calculate Uniform Clearing Price (MCP)
    /// Returns [clearing_price, clearing_volume]
    pub fn calculate_clearing_price(&self) -> Vec<f64> {
        let mut bids: Vec<&AuctionOrderWasm> = self.orders.iter().filter(|o| o.is_bid).collect();
        let mut asks: Vec<&AuctionOrderWasm> = self.orders.iter().filter(|o| !o.is_bid).collect();

        // Sort Bids DESC
        bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(Ordering::Equal));
        // Sort Asks ASC
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(Ordering::Equal));

        let mut clearing_price = 0.0;
        let mut max_volume = 0.0;

        // Collect all unique price points
        let mut prices: Vec<f64> = self.orders.iter().map(|o| o.price).collect();
        prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        prices.dedup();
        
        for &p in &prices {
            let supply: f64 = asks.iter()
                .filter(|o| o.price <= p)
                .map(|o| o.amount)
                .sum();
                
            let demand: f64 = bids.iter()
                .filter(|o| o.price >= p)
                .map(|o| o.amount)
                .sum();
                
            let volume = supply.min(demand);
            
            if volume > max_volume {
                max_volume = volume;
                clearing_price = p;
            } else if (volume - max_volume).abs() < f64::EPSILON && volume > 0.0 {
                 // Maximizing producer surplus
                 clearing_price = p;
            }
        }

        vec![clearing_price, max_volume]
    }
}

// ============================================================================
// FFI Exports for Manual Bridge (no-std / manual wasm)
// ============================================================================

static mut AUCTION: Option<AuctionSimulator> = None;
static mut RESULT_BUFFER: [f64; 2] = [0.0; 2];

#[no_mangle]
pub extern "C" fn auction_init() {
    unsafe {
        AUCTION = Some(AuctionSimulator::new());
    }
}

#[no_mangle]
pub extern "C" fn auction_add_order(id: u32, price: f64, amount: f64, is_bid: u32) {
    unsafe {
        if let Some(auction) = AUCTION.as_mut() {
            auction.add_order(id, price, amount, is_bid != 0);
        }
    }
}

#[no_mangle]
pub extern "C" fn auction_clear() {
    unsafe {
        if let Some(auction) = AUCTION.as_mut() {
            auction.clear();
        }
    }
}

#[no_mangle]
pub extern "C" fn auction_calculate_clearing_price() -> *const f64 {
    unsafe {
        if let Some(auction) = AUCTION.as_ref() {
            let res = auction.calculate_clearing_price();
            if res.len() >= 2 {
                RESULT_BUFFER[0] = res[0];
                RESULT_BUFFER[1] = res[1];
            }
        }
        RESULT_BUFFER.as_ptr()
    }
}
