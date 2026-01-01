//! Options Pricing Module
//! 
//! Black-Scholes pricing and Greeks calculations.

use std::f64::consts::PI;

const R: f64 = 0.0;
const SIGMA: f64 = 0.5;

fn normal_cdf(z: f64) -> f64 {
    const BETA1: f64 = -0.0004406;
    const BETA2: f64 = 0.0418198;
    const BETA3: f64 = 0.9;
    let exponent = -PI.sqrt() * (BETA1 * z.powi(5) + BETA2 * z.powi(3) + BETA3 * z);
    1.0 / (1.0 + exponent.exp())
}

fn normal_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * PI).sqrt()
}

fn calc_d1(s: f64, k: f64, t: f64) -> f64 {
    ((s / k).ln() + (R + 0.5 * SIGMA * SIGMA) * t) / (SIGMA * t.sqrt())
}

fn calc_d2(d1: f64, t: f64) -> f64 {
    d1 - SIGMA * t.sqrt()
}

#[no_mangle]
pub extern "C" fn black_scholes(s: f64, k: f64, t: f64, is_call: u8) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 { return 0.0; }
    
    let d1 = calc_d1(s, k, t);
    let d2 = calc_d2(d1, t);
    
    if is_call == 1 {
        s * normal_cdf(d1) - k * (-R * t).exp() * normal_cdf(d2)
    } else {
        k * (-R * t).exp() * normal_cdf(-d2) - s * normal_cdf(-d1)
    }
}

#[no_mangle]
pub extern "C" fn delta_calc(s: f64, k: f64, t: f64, is_call: u8) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 { return 0.0; }
    let d1 = calc_d1(s, k, t);
    if is_call == 1 { normal_cdf(d1) } else { -normal_cdf(-d1) }
}

#[no_mangle]
pub extern "C" fn gamma_calc(s: f64, k: f64, t: f64) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 { return 0.0; }
    let d1 = calc_d1(s, k, t);
    normal_pdf(d1) / (s * SIGMA * t.sqrt())
}

#[no_mangle]
pub extern "C" fn vega_calc(s: f64, k: f64, t: f64) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 { return 0.0; }
    let d1 = calc_d1(s, k, t);
    s * normal_pdf(d1) * t.sqrt() * 0.01
}

#[no_mangle]
pub extern "C" fn theta_calc(s: f64, k: f64, t: f64, is_call: u8) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 { return 0.0; }
    
    let d1 = calc_d1(s, k, t);
    let d2 = calc_d2(d1, t);
    
    let theta_value = if is_call == 1 {
        (-s * normal_pdf(d1) * SIGMA) / (2.0 * t.sqrt()) - R * k * (-R * t).exp() * normal_cdf(d2)
    } else {
        (-s * normal_pdf(d1) * SIGMA) / (2.0 * t.sqrt()) - R * k * (-R * t).exp() * normal_cdf(-d2)
    };
    
    theta_value / 365.0
}

#[no_mangle]
pub extern "C" fn rho_calc(s: f64, k: f64, t: f64, is_call: u8) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 { return 0.0; }
    
    let d1 = calc_d1(s, k, t);
    let d2 = calc_d2(d1, t);
    
    let rho_value = if is_call == 1 {
        k * t * (-R * t).exp() * normal_cdf(d2)
    } else {
        -k * t * (-R * t).exp() * normal_cdf(-d2)
    };
    
    rho_value * 0.01
}

#[no_mangle]
pub extern "C" fn batch_black_scholes(ptr: *const f64, count: usize, out_ptr: *mut f64) -> usize {
    let input = unsafe { std::slice::from_raw_parts(ptr, count * 4) };
    let output = unsafe { std::slice::from_raw_parts_mut(out_ptr, count) };
    
    for i in 0..count {
        output[i] = black_scholes(
            input[i * 4],
            input[i * 4 + 1],
            input[i * 4 + 2],
            input[i * 4 + 3] as u8
        );
    }
    count
}

#[no_mangle]
pub extern "C" fn calc_all_greeks(s: f64, k: f64, t: f64, is_call: u8, out_ptr: *mut f64) {
    let output = unsafe { std::slice::from_raw_parts_mut(out_ptr, 5) };
    output[0] = delta_calc(s, k, t, is_call);
    output[1] = gamma_calc(s, k, t);
    output[2] = vega_calc(s, k, t);
    output[3] = theta_calc(s, k, t, is_call);
    output[4] = rho_calc(s, k, t, is_call);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const EPSILON: f64 = 0.0001;
    fn approx_eq(a: f64, b: f64) -> bool { (a - b).abs() < EPSILON }
    
    #[test]
    fn test_normal_cdf() {
        assert!(approx_eq(normal_cdf(0.0), 0.5));
        assert!(normal_cdf(3.0) > 0.99);
        assert!(normal_cdf(-3.0) < 0.01);
    }
    
    #[test]
    fn test_black_scholes_call() {
        let price = black_scholes(100.0, 100.0, 1.0, 1);
        assert!(price > 15.0 && price < 25.0);
    }
    
    #[test]
    fn test_black_scholes_put() {
        let price = black_scholes(100.0, 100.0, 1.0, 0);
        let call_price = black_scholes(100.0, 100.0, 1.0, 1);
        assert!(approx_eq(price, call_price));
    }
    
    #[test]
    fn test_greeks() {
        assert!(delta_calc(100.0, 100.0, 1.0, 1) > 0.4);
        assert!(gamma_calc(100.0, 100.0, 1.0) > 0.0);
        assert!(vega_calc(100.0, 100.0, 1.0) > 0.0);
    }
}
