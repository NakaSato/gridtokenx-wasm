use wasm_bindgen::prelude::*;
use std::f64;

const R: f64 = 0.0;
const SIGMA: f64 = 0.5;

#[wasm_bindgen]
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
}

fn normal_cdf(z: f64) -> f64 {
    let beta1 = -0.0004406;
    let beta2 = 0.0418198;
    let beta3 = 0.9;
    let exponent = -f64::consts::PI.sqrt() * (beta1 * z.powi(5) + beta2 * z.powi(3) + beta3 * z);
    1.0 / (1.0 + exponent.exp())
}

fn normal_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * f64::consts::PI).sqrt()
}

#[wasm_bindgen]
pub fn black_scholes(s: f64, k: f64, t: f64, is_call: bool) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    let d1 = ( (s / k).ln() + (R + 0.5 * SIGMA * SIGMA) * t ) / (SIGMA * t.sqrt());
    let d2 = d1 - SIGMA * t.sqrt();
    
    if is_call {
        s * normal_cdf(d1) - k * (-R * t).exp() * normal_cdf(d2)
    } else {
        k * (-R * t).exp() * normal_cdf(-d2) - s * normal_cdf(-d1)
    }
}

#[wasm_bindgen]
pub fn calculate_greeks(s: f64, k: f64, t: f64, is_call: bool) -> Greeks {
    Greeks {
        delta: delta_calc(s, k, t, is_call),
        gamma: gamma_calc(s, k, t),
        vega: vega_calc(s, k, t),
        theta: theta_calc(s, k, t, is_call),
        rho: rho_calc(s, k, t, is_call),
    }
}

#[wasm_bindgen]
pub fn delta_calc(s: f64, k: f64, t: f64, is_call: bool) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    let d1 = ((s / k).ln() + (R + SIGMA.powi(2) / 2.0) * t) / (SIGMA * t.sqrt());
    if is_call {
        normal_cdf(d1)
    } else {
        -normal_cdf(-d1)
    }
}

#[wasm_bindgen]
pub fn gamma_calc(s: f64, k: f64, t: f64) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    let d1 = ((s / k).ln() + (R + SIGMA.powi(2) / 2.0) * t) / (SIGMA * t.sqrt());
    normal_pdf(d1) / (s * SIGMA * t.sqrt())
}

#[wasm_bindgen]
pub fn vega_calc(s: f64, k: f64, t: f64) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    let d1 = ((s / k).ln() + (R + SIGMA.powi(2) / 2.0) * t) / (SIGMA * t.sqrt());
    s * normal_pdf(d1) * t.sqrt() * 0.01
}

#[wasm_bindgen]
pub fn theta_calc(s: f64, k: f64, t: f64, is_call: bool) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    let d1 = ((s / k).ln() + (R + SIGMA.powi(2) / 2.0) * t) / (SIGMA * t.sqrt());
    let d2 = d1 - SIGMA * t.sqrt();
    let theta_value = (-s * normal_pdf(d1) * SIGMA) / (2.0 * t.sqrt()) - R * k * (-R * t).exp() * normal_cdf(if is_call { d2 } else { -d2 });
    theta_value / 365.0
}

#[wasm_bindgen]
pub fn rho_calc(s: f64, k: f64, t: f64, is_call: bool) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    let d1 = ((s / k).ln() + (R + SIGMA.powi(2) / 2.0) * t) / (SIGMA * t.sqrt());
    let d2 = d1 - SIGMA * t.sqrt();
    let val = if is_call { 1.0 } else { -1.0 } * k * t * (-R * t).exp() * normal_cdf(if is_call { d2 } else { -d2 }) * 0.01;
    val
}
