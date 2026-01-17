//! Black-Scholes Options Pricing Module
//! 
//! Calculates option prices and Greeks for energy derivatives.

use wasm_bindgen::prelude::*;
use std::f64::consts::PI;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct OptionResult {
    pub call_price: f64,
    pub put_price: f64,
    pub call_delta: f64,
    pub put_delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub call_theta: f64,
    pub put_theta: f64,
    pub call_rho: f64,
    pub put_rho: f64,
}

/// Cumulative Normal Distribution Function
fn cndf(x: f64) -> f64 {
    let a1 = 0.319381530;
    let a2 = -0.356563782;
    let a3 = 1.781477937;
    let a4 = -1.821255978;
    let a5 = 1.330274429;
    let l = x.abs();
    let k = 1.0 / (1.0 + 0.2316419 * l);
    let w = 1.0 - 1.0 / (2.0 * PI).sqrt() * (-l * l / 2.0).exp() *
            (a1 * k + a2 * k * k + a3 * k.powi(3) + a4 * k.powi(4) + a5 * k.powi(5));

    if x < 0.0 {
        1.0 - w
    } else {
        w
    }
}

/// Normal Probability Density Function
fn npdf(x: f64) -> f64 {
    (1.0 / (2.0 * PI).sqrt()) * (-x * x / 2.0).exp()
}

/// Calculate Black-Scholes price and Greeks
/// s: spot price
/// k: strike price
/// t: time to maturity (years)
/// r: risk-free rate
/// v: volatility
#[wasm_bindgen]
pub fn calculate_black_scholes(s: f64, k: f64, t: f64, r: f64, v: f64) -> Result<JsValue, JsValue> {
    if t <= 0.0 {
        // Expired
        let val = if s > k { s - k } else { 0.0 };
        return Ok(serde_wasm_bindgen::to_value(&OptionResult {
            call_price: val,
            put_price: if k > s { k - s } else { 0.0 },
            call_delta: if s > k { 1.0 } else { 0.0 },
            put_delta: if k > s { -1.0 } else { 0.0 },
            gamma: 0.0,
            vega: 0.0,
            call_theta: 0.0,
            put_theta: 0.0,
            call_rho: 0.0,
            put_rho: 0.0,
        })?);
    }

    let d1 = ((s / k).ln() + (r + v * v / 2.0) * t) / (v * t.sqrt());
    let d2 = d1 - v * t.sqrt();

    let nd1 = cndf(d1);
    let nd2 = cndf(d2);
    let n_prime_d1 = npdf(d1);

    let call_price = s * nd1 - k * (-r * t).exp() * nd2;
    let put_price = k * (-r * t).exp() * cndf(-d2) - s * cndf(-d1);

    // Greeks
    let call_delta = nd1;
    let put_delta = nd1 - 1.0;

    let gamma = n_prime_d1 / (s * v * t.sqrt());

    let vega = s * t.sqrt() * n_prime_d1 / 100.0; // Scaled for 1% change

    let theta_common = -(s * n_prime_d1 * v) / (2.0 * t.sqrt());
    let call_theta = (theta_common - r * k * (-r * t).exp() * nd2) / 365.0;
    let put_theta = (theta_common + r * k * (-r * t).exp() * cndf(-d2)) / 365.0;

    let call_rho = k * t * (-r * t).exp() * nd2 / 100.0;
    let put_rho = -k * t * (-r * t).exp() * cndf(-d2) / 100.0;

    Ok(serde_wasm_bindgen::to_value(&OptionResult {
        call_price,
        put_price,
        call_delta,
        put_delta,
        gamma,
        vega,
        call_theta,
        put_theta,
        call_rho,
        put_rho,
    })?)
}
