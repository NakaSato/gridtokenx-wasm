#![allow(static_mut_refs)]
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

    /// Calculate Uniform Clearing Price (MCP) - Optimized to O(n log n)
    /// Returns [clearing_price, clearing_volume]
    pub fn calculate_clearing_price(&self) -> Vec<f64> {
        let mut bids: Vec<&AuctionOrderWasm> = self.orders.iter().filter(|o| o.is_bid).collect();
        let mut asks: Vec<&AuctionOrderWasm> = self.orders.iter().filter(|o| !o.is_bid).collect();

        if bids.is_empty() || asks.is_empty() {
            return vec![0.0, 0.0];
        }

        // Sort Bids DESC by price
        bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(Ordering::Equal));
        // Sort Asks ASC by price
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(Ordering::Equal));

        // Build cumulative demand curve (descending from highest bid)
        let mut cum_demand: Vec<(f64, f64)> = Vec::with_capacity(bids.len()); // (price, cumulative_amount)
        let mut cum_amount = 0.0;
        for bid in &bids {
            cum_amount += bid.amount;
            cum_demand.push((bid.price, cum_amount));
        }

        // Build cumulative supply curve (ascending from lowest ask)
        let mut cum_supply: Vec<(f64, f64)> = Vec::with_capacity(asks.len()); // (price, cumulative_amount)
        let mut cum_amount = 0.0;
        for ask in &asks {
            cum_amount += ask.amount;
            cum_supply.push((ask.price, cum_amount));
        }

        // Find maximum intersection using two-pointer technique - O(n)
        let mut clearing_price = 0.0;
        let mut max_volume = 0.0;
        let mut bid_idx = 0;
        let mut ask_idx = 0;

        while bid_idx < cum_demand.len() && ask_idx < cum_supply.len() {
            let (bid_price, demand) = cum_demand[bid_idx];
            let (ask_price, supply) = cum_supply[ask_idx];

            if ask_price <= bid_price {
                // Potential match at this price level
                let volume = supply.min(demand);
                
                if volume > max_volume {
                    max_volume = volume;
                    // Use midpoint as clearing price
                    clearing_price = (ask_price + bid_price) / 2.0;
                } else if (volume - max_volume).abs() < f64::EPSILON && volume > 0.0 {
                    // Same volume, update to higher price (producer surplus)
                    clearing_price = (ask_price + bid_price) / 2.0;
                }
                
                // Move to next supply level
                ask_idx += 1;
            } else {
                // Price gap, move to lower demand
                bid_idx += 1;
            }
        }

        vec![clearing_price, max_volume]
    }
}

// ============================================================================
// FFI Exports for Manual Bridge (no-std / manual wasm)
// FFI Exports for Manual Bridge (no-std / manual wasm)
// ============================================================================

#[allow(static_mut_refs)]
static mut AUCTION: Option<AuctionSimulator> = None;
#[allow(static_mut_refs)]
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
