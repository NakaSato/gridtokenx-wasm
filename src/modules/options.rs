use std::f64::consts::PI;

const R: f64 = 0.0;      // Risk-free rate
const SIGMA: f64 = 0.5;  // Volatility

/// Normal cumulative distribution function (CDF)
/// Uses the same approximation as the TypeScript version
fn normal_cdf(z: f64) -> f64 {
    const BETA1: f64 = -0.0004406;
    const BETA2: f64 = 0.0418198;
    const BETA3: f64 = 0.9;
    
    let exponent = -PI.sqrt() * (BETA1 * z.powi(5) + BETA2 * z.powi(3) + BETA3 * z);
    
    1.0 / (1.0 + exponent.exp())
}

/// Normal probability density function (PDF)
fn normal_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * PI).sqrt()
}

/// Calculate d1 parameter for Black-Scholes formula
fn calc_d1(s: f64, k: f64, t: f64) -> f64 {
    (s / k).ln() + (R + 0.5 * SIGMA * SIGMA) * t / (SIGMA * t.sqrt())
}

/// Calculate d2 parameter for Black-Scholes formula
fn calc_d2(d1: f64, t: f64) -> f64 {
    d1 - SIGMA * t.sqrt()
}

/// Black-Scholes option pricing
/// s = current price
/// k = strike price
/// t = time to expiration (in years, or as fraction)
/// is_call = 1 for call, 0 for put
#[no_mangle]
pub extern "C" fn black_scholes(s: f64, k: f64, t: f64, is_call: u8) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    
    let d1 = calc_d1(s, k, t);
    let d2 = calc_d2(d1, t);
    
    let nd1 = normal_cdf(d1);
    let nd2 = normal_cdf(d2);
    let n_neg_d1 = normal_cdf(-d1);
    let n_neg_d2 = normal_cdf(-d2);
    
    if is_call == 1 {
        s * nd1 - k * (-R * t).exp() * nd2
    } else {
        k * (-R * t).exp() * n_neg_d2 - s * n_neg_d1
    }
}

/// Delta: rate of change of option price with respect to underlying price
#[no_mangle]
pub extern "C" fn delta_calc(s: f64, k: f64, t: f64, is_call: u8) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    
    let d1 = calc_d1(s, k, t);
    
    if is_call == 1 {
        normal_cdf(d1)
    } else {
        -normal_cdf(-d1)
    }
}

/// Gamma: rate of change of delta with respect to underlying price
#[no_mangle]
pub extern "C" fn gamma_calc(s: f64, k: f64, t: f64) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    
    let d1 = calc_d1(s, k, t);
    normal_pdf(d1) / (s * SIGMA * t.sqrt())
}

/// Vega: sensitivity to volatility (per 1% change)
#[no_mangle]
pub extern "C" fn vega_calc(s: f64, k: f64, t: f64) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    
    let d1 = calc_d1(s, k, t);
    s * normal_pdf(d1) * t.sqrt() * 0.01
}

/// Theta: time decay (per day)
#[no_mangle]
pub extern "C" fn theta_calc(s: f64, k: f64, t: f64, is_call: u8) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    
    let d1 = calc_d1(s, k, t);
    let d2 = calc_d2(d1, t);
    
    let theta_value = if is_call == 1 {
        (-s * normal_pdf(d1) * SIGMA) / (2.0 * t.sqrt()) 
            - R * k * (-R * t).exp() * normal_cdf(d2)
    } else {
        (-s * normal_pdf(d1) * SIGMA) / (2.0 * t.sqrt()) 
            - R * k * (-R * t).exp() * normal_cdf(-d2)
    };
    
    theta_value / 365.0
}

/// Rho: sensitivity to interest rate (per 1% change)
#[no_mangle]
pub extern "C" fn rho_calc(s: f64, k: f64, t: f64, is_call: u8) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    
    let d1 = calc_d1(s, k, t);
    let d2 = calc_d2(d1, t);
    
    let rho_value = if is_call == 1 {
        k * t * (-R * t).exp() * normal_cdf(d2)
    } else {
        -k * t * (-R * t).exp() * normal_cdf(-d2)
    };
    
    rho_value * 0.01
}

/// Batch Black-Scholes calculation for multiple options
/// Input buffer format: [s, k, t, is_call, s, k, t, is_call, ...]
/// Output buffer format: [price, price, ...]
/// Returns number of prices calculated
#[no_mangle]
pub extern "C" fn batch_black_scholes(ptr: *const f64, count: usize, out_ptr: *mut f64) -> usize {
    let input = unsafe { std::slice::from_raw_parts(ptr, count * 4) };
    let output = unsafe { std::slice::from_raw_parts_mut(out_ptr, count) };
    
    for i in 0..count {
        let s = input[i * 4];
        let k = input[i * 4 + 1];
        let t = input[i * 4 + 2];
        let is_call = input[i * 4 + 3] as u8;
        
        output[i] = black_scholes(s, k, t, is_call);
    }
    
    count
}

/// Batch Greeks calculation - returns all Greeks for one option
/// Output format: [delta, gamma, vega, theta, rho]
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
    
    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }
    
    #[test]
    fn test_normal_cdf() {
        // CDF(0) should be approximately 0.5
        assert!(approx_eq(normal_cdf(0.0), 0.5));
        // CDF of large positive should be close to 1
        assert!(normal_cdf(3.0) > 0.99);
        // CDF of large negative should be close to 0
        assert!(normal_cdf(-3.0) < 0.01);
    }
    
    #[test]
    fn test_normal_pdf() {
        // PDF at 0 should be maximum
        let pdf_0 = normal_pdf(0.0);
        assert!(pdf_0 > normal_pdf(1.0));
        assert!(pdf_0 > normal_pdf(-1.0));
    }
    
    #[test]
    fn test_black_scholes_call() {
        // Test with: s=100, k=100, t=1.0 (1 year), call
        let price = black_scholes(100.0, 100.0, 1.0, 1);
        // With sigma=0.5, r=0, ATM call should have significant value
        assert!(price > 15.0 && price < 25.0);
    }
    
    #[test]
    fn test_black_scholes_put() {
        // Test with: s=100, k=100, t=1.0 (1 year), put
        let price = black_scholes(100.0, 100.0, 1.0, 0);
        // ATM put with r=0 should equal call (put-call parity)
        let call_price = black_scholes(100.0, 100.0, 1.0, 1);
        assert!(approx_eq(price, call_price));
    }
    
    #[test]
    fn test_delta_call() {
        // ATM call delta should be around 0.5
        let delta = delta_calc(100.0, 100.0, 1.0, 1);
        assert!(delta > 0.4 && delta < 0.7);
    }
    
    #[test]
    fn test_delta_put() {
        // ATM put delta should be negative and around -0.5
        let delta = delta_calc(100.0, 100.0, 1.0, 0);
        assert!(delta < -0.3 && delta > -0.7);
    }
    
    #[test]
    fn test_gamma_positive() {
        // Gamma should always be positive
        let gamma = gamma_calc(100.0, 100.0, 1.0);
        assert!(gamma > 0.0);
    }
    
    #[test]
    fn test_vega_positive() {
        // Vega should always be positive
        let vega = vega_calc(100.0, 100.0, 1.0);
        assert!(vega > 0.0);
    }
    
    #[test]
    fn test_edge_cases() {
        // Zero time should return 0
        assert_eq!(black_scholes(100.0, 100.0, 0.0, 1), 0.0);
        // Zero price should return 0
        assert_eq!(black_scholes(0.0, 100.0, 1.0, 1), 0.0);
        // Zero strike should return 0
        assert_eq!(black_scholes(100.0, 0.0, 1.0, 1), 0.0);
    }
}
