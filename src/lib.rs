//! GridTokenX WASM Module
//! 
//! High-performance WASM functions for:
//! - Geo clustering (marker clustering for maps)
//! - Bezier curve generation
//! - Energy simulation
//! - Black-Scholes options pricing
//! - Greeks calculations  
//! - P&L chart generation

use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::f64::consts::PI;

// =============================================================================
// STATE MANAGEMENT - Using UnsafeCell for safe mutable statics
// =============================================================================

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
    id: u32,
    lat: f64,
    lng: f64,
}

#[derive(Clone, Copy)]
struct SimulationNode {
    node_type: u8,
    base_value: f64,
    current_value: f64,
    status: u8,
    is_real: u8,
}

#[derive(Clone, Copy)]
struct SimulationFlow {
    flow_index: u32,
    base_power: f64,
    current_power: f64,
}

/// Consolidated WASM state - all mutable buffers in one place
struct WasmState {
    // General buffer for input/output
    buffer: [f64; 20000],
    // Geo clustering
    points: Vec<Point>,
    output_buffer: Vec<f64>,
    // Simulation
    sim_nodes: Vec<SimulationNode>,
    sim_flows: Vec<SimulationFlow>,
    sim_node_output: Vec<f64>,
    sim_flow_output: Vec<f64>,
    rng_state: u32,
    // P&L
    pnl_buffer: [f64; 1000],
}

impl WasmState {
    const fn new() -> Self {
        WasmState {
            buffer: [0.0; 20000],
            points: Vec::new(),
            output_buffer: Vec::new(),
            sim_nodes: Vec::new(),
            sim_flows: Vec::new(),
            sim_node_output: Vec::new(),
            sim_flow_output: Vec::new(),
            rng_state: 12345,
            pnl_buffer: [0.0; 1000],
        }
    }
}

/// Wrapper type to make UnsafeCell Sync
/// # Safety
/// This is safe because WASM is single-threaded
struct SyncState(UnsafeCell<WasmState>);
unsafe impl Sync for SyncState {}

/// Global state wrapped for interior mutability
static STATE: SyncState = SyncState(UnsafeCell::new(WasmState::new()));

/// Get mutable reference to global state
/// # Safety
/// This is safe in single-threaded WASM context
#[inline]
fn state() -> &'static mut WasmState {
    unsafe { &mut *STATE.0.get() }
}

// =============================================================================
// GEO CLUSTERING
// =============================================================================

fn lng_to_x(lng: f64) -> f64 {
    (lng + 180.0) / 360.0
}

fn lat_to_y(lat: f64) -> f64 {
    let sin_lat = (lat * PI / 180.0).sin();
    let y = 0.5 - (0.25 * ((1.0 + sin_lat) / (1.0 - sin_lat)).ln() / PI);
    y.clamp(0.0, 1.0)
}

#[no_mangle]
pub extern "C" fn load_points(ptr: *const f64, count: usize) {
    let s = state();
    s.points.clear();
    let input = unsafe { std::slice::from_raw_parts(ptr, count * 3) };
    
    for i in 0..count {
        let lat = input[i * 3];
        let lng = input[i * 3 + 1];
        let id = input[i * 3 + 2] as u32;
        
        s.points.push(Point {
            x: lng_to_x(lng),
            y: lat_to_y(lat),
            id,
            lat,
            lng,
        });
    }
}

struct ClusterData {
    sum_x: f64,
    sum_y: f64,
    sum_lat: f64,
    sum_lng: f64,
    count: u32,
    first_id: u32,
}

#[no_mangle]
pub extern "C" fn get_clusters(
    min_lng: f64, min_lat: f64,
    max_lng: f64, max_lat: f64,
    zoom: f64
) -> usize {
    let s = state();
    s.output_buffer.clear();
    
    let min_x = lng_to_x(min_lng);
    let max_x = lng_to_x(max_lng);
    let min_y = lat_to_y(max_lat);
    let max_y = lat_to_y(min_lat);

    let radius = 60.0;
    let cells = (2.0f64.powf(zoom) * (256.0 / radius)).ceil();
    
    let mut grid: HashMap<(i32, i32), ClusterData> = HashMap::new();

    for point in &s.points {
        if point.x < min_x || point.x > max_x || point.y < min_y || point.y > max_y {
            continue;
        }
        
        let grid_x = (point.x * cells) as i32;
        let grid_y = (point.y * cells) as i32;
        
        let entry = grid.entry((grid_x, grid_y)).or_insert(ClusterData {
            sum_x: 0.0,
            sum_y: 0.0,
            sum_lat: 0.0,
            sum_lng: 0.0,
            count: 0,
            first_id: point.id,
        });
        
        entry.sum_x += point.x;
        entry.sum_y += point.y;
        entry.sum_lat += point.lat;
        entry.sum_lng += point.lng;
        entry.count += 1;
    }
    
    for data in grid.values() {
        let count_f = data.count as f64;
        let avg_lat = data.sum_lat / count_f;
        let avg_lng = data.sum_lng / count_f;
        
        s.output_buffer.push(avg_lat);
        s.output_buffer.push(avg_lng);
        s.output_buffer.push(count_f);
        s.output_buffer.push(data.first_id as f64);
    }
    
    s.output_buffer.len() / 4
}

#[no_mangle]
pub extern "C" fn get_buffer_ptr() -> *mut f64 {
    state().buffer.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn get_output_buffer_ptr() -> *const f64 {
    state().output_buffer.as_ptr()
}

// =============================================================================
// BEZIER CURVES
// =============================================================================

#[no_mangle]
pub extern "C" fn calculate_bezier(
    x1: f64, y1: f64,
    x2: f64, y2: f64,
    curve_intensity: f64,
    segments: usize,
    ptr: *mut f64
) -> usize {
    let buffer = unsafe { std::slice::from_raw_parts_mut(ptr, (segments + 1) * 2) };

    let mid_x = (x1 + x2) / 2.0;
    let mid_y = (y1 + y2) / 2.0;

    let dx = x2 - x1;
    let dy = y2 - y1;
    let distance = (dx * dx + dy * dy).sqrt();

    let (perp_x, perp_y) = if distance > 0.0 {
        (-dy / distance, dx / distance)
    } else {
        (0.0, 0.0)
    };

    let sum_coords = (x1 + y1) as i64;
    let direction = if sum_coords % 2 == 0 { 1.0 } else { -1.0 };
    let offset = distance * curve_intensity * direction;

    let cx = mid_x + perp_x * offset;
    let cy = mid_y + perp_y * offset;

    for i in 0..=segments {
        let t = i as f64 / segments as f64;
        let one_minus_t = 1.0 - t;

        let x = one_minus_t * one_minus_t * x1 + 2.0 * one_minus_t * t * cx + t * t * x2;
        let y = one_minus_t * one_minus_t * y1 + 2.0 * one_minus_t * t * cy + t * t * y2;

        buffer[i * 2] = x;
        buffer[i * 2 + 1] = y;
    }

    segments + 1
}

// =============================================================================
// ENERGY SIMULATION
// =============================================================================

fn rand_float() -> f64 {
    let s = state();
    s.rng_state = s.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
    (s.rng_state as f64) / (u32::MAX as f64)
}

fn fluctuate(base_value: f64, percent_range: f64) -> f64 {
    let variance = base_value * (percent_range / 100.0);
    let rand = rand_float() * 2.0 - 1.0;
    base_value + rand * variance
}

fn get_time_multiplier(hour: f64, node_type: u8) -> f64 {
    let h = hour;
    if node_type == 0 {
        if h >= 19.0 || h < 6.0 { return 0.05; }
        if h >= 6.0 && h < 8.0 { return 0.3; }
        if h >= 8.0 && h < 10.0 { return 0.6; }
        if h >= 10.0 && h < 12.0 { return 0.85; }
        if h >= 12.0 && h < 14.0 { return 1.0; }
        if h >= 14.0 && h < 16.0 { return 0.9; }
        if h >= 16.0 && h < 18.0 { return 0.6; }
        if h >= 18.0 && h < 19.0 { return 0.2; }
        return 0.5;
    } else if node_type == 2 {
        if h >= 0.0 && h < 6.0 { return 0.2; }
        if h >= 6.0 && h < 8.0 { return 0.5; }
        if h >= 8.0 && h < 10.0 { return 0.9; }
        if h >= 10.0 && h < 12.0 { return 0.8; }
        if h >= 12.0 && h < 14.0 { return 0.6; }
        if h >= 14.0 && h < 17.0 { return 0.85; }
        if h >= 17.0 && h < 20.0 { return 1.0; }
        if h >= 20.0 && h < 22.0 { return 0.7; }
        if h >= 22.0 { return 0.3; }
        return 0.5;
    } else if node_type == 1 {
        if h >= 19.0 || h < 6.0 { return 0.4; }
        if h >= 6.0 && h < 10.0 { return 0.5; }
        if h >= 10.0 && h < 14.0 { return 0.85; }
        if h >= 14.0 && h < 17.0 { return 0.95; }
        if h >= 17.0 && h < 19.0 { return 0.7; }
        return 0.6;
    }
    1.0
}

#[no_mangle]
pub extern "C" fn init_simulation_nodes(ptr: *const f64, count: usize) {
    let s = state();
    s.sim_nodes.clear();
    let input = unsafe { std::slice::from_raw_parts(ptr, count * 5) };
    
    for i in 0..count {
        s.sim_nodes.push(SimulationNode {
            node_type: input[i * 5] as u8,
            base_value: input[i * 5 + 1],
            current_value: input[i * 5 + 2],
            status: input[i * 5 + 3] as u8,
            is_real: input[i * 5 + 4] as u8,
        });
    }
}

#[no_mangle]
pub extern "C" fn init_simulation_flows(ptr: *const f64, count: usize) {
    let s = state();
    s.sim_flows.clear();
    let input = unsafe { std::slice::from_raw_parts(ptr, count * 2) };
    
    for i in 0..count {
        s.sim_flows.push(SimulationFlow {
            flow_index: i as u32,
            base_power: input[i * 2],
            current_power: input[i * 2 + 1],
        });
    }
}

#[no_mangle]
pub extern "C" fn update_simulation(hour: f64, minute: f64) {
    let s = state();
    s.sim_node_output.clear();
    s.sim_flow_output.clear();
    
    let minute_variation = (minute / 60.0 * PI * 2.0).sin() * 0.05;
    
    for node in s.sim_nodes.iter_mut() {
        if node.is_real == 1 {
            s.sim_node_output.push(node.current_value);
            s.sim_node_output.push(node.status as f64);
            continue;
        }
        
        let multiplier = get_time_multiplier(hour, node.node_type);
        let base_calculated = node.base_value * multiplier * (1.0 + minute_variation);
        let new_value = fluctuate(base_calculated, 8.0).max(0.0);
        
        if rand_float() < 0.005 {
            node.status = if rand_float() > 0.5 { 1 } else { 0 };
        }
        
        node.current_value = new_value;
        s.sim_node_output.push(new_value);
        s.sim_node_output.push(node.status as f64);
    }
    
    let gen_multiplier = get_time_multiplier(hour, 0);
    for flow in s.sim_flows.iter_mut() {
        let base = flow.base_power * gen_multiplier;
        let new_power = fluctuate(base, 12.0).max(50.0);
        flow.current_power = new_power;
        s.sim_flow_output.push(new_power);
    }
}

#[no_mangle]
pub extern "C" fn get_node_output_ptr() -> *const f64 {
    state().sim_node_output.as_ptr()
}

#[no_mangle]
pub extern "C" fn get_flow_output_ptr() -> *const f64 {
    state().sim_flow_output.as_ptr()
}

// =============================================================================
// OPTIONS PRICING (Black-Scholes)
// =============================================================================

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

#[no_mangle]
pub extern "C" fn delta_calc(s: f64, k: f64, t: f64, is_call: u8) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    let d1 = calc_d1(s, k, t);
    if is_call == 1 { normal_cdf(d1) } else { -normal_cdf(-d1) }
}

#[no_mangle]
pub extern "C" fn gamma_calc(s: f64, k: f64, t: f64) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    let d1 = calc_d1(s, k, t);
    normal_pdf(d1) / (s * SIGMA * t.sqrt())
}

#[no_mangle]
pub extern "C" fn vega_calc(s: f64, k: f64, t: f64) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    let d1 = calc_d1(s, k, t);
    s * normal_pdf(d1) * t.sqrt() * 0.01
}

#[no_mangle]
pub extern "C" fn theta_calc(s: f64, k: f64, t: f64, is_call: u8) -> f64 {
    if t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return 0.0;
    }
    
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

// =============================================================================
// P&L CHART
// =============================================================================

#[no_mangle]
pub extern "C" fn calculate_pnl(
    price: f64,
    strike_price: f64,
    premium: f64,
    contract_type: u8,
    position_type: u8
) -> f64 {
    if contract_type == 0 {
        if position_type == 0 {
            (price - strike_price).max(0.0) - premium
        } else {
            premium - (price - strike_price).max(0.0)
        }
    } else {
        if position_type == 0 {
            (strike_price - price).max(0.0) - premium
        } else {
            premium - (strike_price - price).max(0.0)
        }
    }
}

#[no_mangle]
pub extern "C" fn get_pnl_buffer_ptr() -> *mut f64 {
    state().pnl_buffer.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn generate_pnl_batch(
    strike_price: f64,
    premium: f64,
    contract_type: u8,
    position_type: u8,
    range_percent: f64,
    num_points: usize
) -> usize {
    let num_points = num_points.min(500);
    
    if num_points == 0 || strike_price <= 0.0 {
        return 0;
    }
    
    let range = strike_price * range_percent;
    let min_price = strike_price - range;
    let max_price = strike_price + range;
    let price_step = (max_price - min_price) / (num_points as f64 - 1.0);
    
    let s = state();
    for i in 0..num_points {
        let price = min_price + (i as f64) * price_step;
        let pnl = calculate_pnl(price, strike_price, premium, contract_type, position_type);
        
        s.pnl_buffer[i * 2] = price;
        s.pnl_buffer[i * 2 + 1] = pnl;
    }
    
    num_points
}

#[no_mangle]
pub extern "C" fn get_pnl_range(num_points: usize, out_ptr: *mut f64) {
    let output = unsafe { std::slice::from_raw_parts_mut(out_ptr, 2) };
    let s = state();
    
    let mut min_pnl = f64::MAX;
    let mut max_pnl = f64::MIN;
    
    for i in 0..num_points.min(500) {
        let pnl = s.pnl_buffer[i * 2 + 1];
        if pnl < min_pnl { min_pnl = pnl; }
        if pnl > max_pnl { max_pnl = pnl; }
    }
    
    output[0] = min_pnl;
    output[1] = max_pnl;
}

// =============================================================================
// UNIT TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    const EPSILON: f64 = 0.0001;
    
    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }
    
    #[test]
    fn test_normal_cdf() {
        assert!(approx_eq(normal_cdf(0.0), 0.5));
        assert!(normal_cdf(3.0) > 0.99);
        assert!(normal_cdf(-3.0) < 0.01);
    }
    
    #[test]
    fn test_normal_pdf() {
        let pdf_0 = normal_pdf(0.0);
        assert!(pdf_0 > normal_pdf(1.0));
        assert!(pdf_0 > normal_pdf(-1.0));
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
    fn test_delta_call() {
        let delta = delta_calc(100.0, 100.0, 1.0, 1);
        assert!(delta > 0.4 && delta < 0.7);
    }
    
    #[test]
    fn test_delta_put() {
        let delta = delta_calc(100.0, 100.0, 1.0, 0);
        assert!(delta < -0.3 && delta > -0.7);
    }
    
    #[test]
    fn test_gamma_positive() {
        let gamma = gamma_calc(100.0, 100.0, 1.0);
        assert!(gamma > 0.0);
    }
    
    #[test]
    fn test_vega_positive() {
        let vega = vega_calc(100.0, 100.0, 1.0);
        assert!(vega > 0.0);
    }
    
    #[test]
    fn test_edge_cases() {
        assert_eq!(black_scholes(100.0, 100.0, 0.0, 1), 0.0);
        assert_eq!(black_scholes(0.0, 100.0, 1.0, 1), 0.0);
        assert_eq!(black_scholes(100.0, 0.0, 1.0, 1), 0.0);
    }
    
    #[test]
    fn test_pnl_long_call_itm() {
        let pnl = calculate_pnl(110.0, 100.0, 5.0, 0, 0);
        assert!(approx_eq(pnl, 5.0));
    }
    
    #[test]
    fn test_pnl_long_call_otm() {
        let pnl = calculate_pnl(90.0, 100.0, 5.0, 0, 0);
        assert!(approx_eq(pnl, -5.0));
    }
    
    #[test]
    fn test_pnl_short_call_itm() {
        let pnl = calculate_pnl(110.0, 100.0, 5.0, 0, 1);
        assert!(approx_eq(pnl, -5.0));
    }
    
    #[test]
    fn test_pnl_long_put_itm() {
        let pnl = calculate_pnl(90.0, 100.0, 5.0, 1, 0);
        assert!(approx_eq(pnl, 5.0));
    }
    
    #[test]
    fn test_pnl_short_put_itm() {
        let pnl = calculate_pnl(90.0, 100.0, 5.0, 1, 1);
        assert!(approx_eq(pnl, -5.0));
    }
    
    #[test]
    fn test_pnl_batch_generation() {
        let count = generate_pnl_batch(100.0, 5.0, 0, 0, 0.2, 10);
        assert_eq!(count, 10);
        
        let s = state();
        let first_price = s.pnl_buffer[0];
        let last_price = s.pnl_buffer[18];
        assert!(first_price < last_price);
        assert!(first_price >= 80.0);
        assert!(last_price <= 120.0);
    }
}
