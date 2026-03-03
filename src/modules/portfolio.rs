use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub symbol: String,
    pub size: f64,
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
    pub pnl: f64,
}

#[derive(Serialize, Deserialize)]
pub struct PortfolioRisk {
    pub total_delta: f64,
    pub total_gamma: f64,
    pub total_vega: f64,
    pub total_theta: f64,
    pub total_rho: f64,
    pub total_pnl: f64,
    pub net_exposure: f64,
}

#[wasm_bindgen]
pub fn calculate_portfolio_risk(positions_js: JsValue) -> Result<JsValue, JsValue> {
    let positions: Vec<PortfolioPosition> = serde_wasm_bindgen::from_value(positions_js)?;
    
    let mut total_delta = 0.0;
    let mut total_gamma = 0.0;
    let mut total_vega = 0.0;
    let mut total_theta = 0.0;
    let mut total_rho = 0.0;
    let mut total_pnl = 0.0;
    let mut net_exposure = 0.0;

    for pos in positions {
        total_delta += pos.delta * pos.size;
        total_gamma += pos.gamma * pos.size;
        total_vega += pos.vega * pos.size;
        total_theta += pos.theta * pos.size;
        total_rho += pos.rho * pos.size;
        total_pnl += pos.pnl;
        net_exposure += pos.size; // Simplified net exposure
    }

    let result = PortfolioRisk {
        total_delta,
        total_gamma,
        total_vega,
        total_theta,
        total_rho,
        total_pnl,
        net_exposure,
    };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}
