use std::collections::HashMap;

#[derive(Clone, Copy)]
struct Point {
    x: f64, // Web Mercator X (0..1)
    y: f64, // Web Mercator Y (0..1)
    id: u32,
    lat: f64,
    lng: f64,
}

static mut POINTS: Vec<Point> = Vec::new();
static mut OUTPUT_BUFFER: Vec<f64> = Vec::new();

// Web Mercator projection helpers
fn lng_to_x(lng: f64) -> f64 {
    (lng + 180.0) / 360.0
}

fn lat_to_y(lat: f64) -> f64 {
    let sin_lat = (lat * std::f64::consts::PI / 180.0).sin();
    let y = 0.5 - (0.25 * ((1.0 + sin_lat) / (1.0 - sin_lat)).ln() / std::f64::consts::PI);
    y.clamp(0.0, 1.0)
}

#[no_mangle]
pub extern "C" fn load_points(ptr: *const f64, count: usize) {
    unsafe {
        POINTS.clear();
        let input = std::slice::from_raw_parts(ptr, count * 3);
        
        for i in 0..count {
            let lat = input[i * 3];
            let lng = input[i * 3 + 1];
            let id = input[i * 3 + 2] as u32; // Assuming ID is passed as f64 for simplicity in array
            
            POINTS.push(Point {
                x: lng_to_x(lng),
                y: lat_to_y(lat),
                id,
                lat,
                lng,
            });
        }
    }
}

struct ClusterData {
    sum_x: f64,
    sum_y: f64,
    sum_lat: f64,
    sum_lng: f64,
    count: u32,
    first_id: u32, // To track the ID if it's a single point
}

#[no_mangle]
pub extern "C" fn get_clusters(
    min_lng: f64, min_lat: f64,
    max_lng: f64, max_lat: f64,
    zoom: f64
) -> usize {
    unsafe {
        OUTPUT_BUFFER.clear();
        
        // Convert bounds to Mercator
        let min_x = lng_to_x(min_lng);
        let max_x = lng_to_x(max_lng);
        let min_y = lat_to_y(max_lat); // Y is flipped in Mercator (0 at top)
        let max_y = lat_to_y(min_lat);

        // Grid size calculations
        // World size is 1.0. At zoom Z, we have roughly 2^Z tiles.
        // We want a cluster radius of approx 40-60px. Tile is 256px.
        // Grid cells per world dimension ~= 2^zoom * (256/radius).
        let radius = 60.0;
        let cells = (2.0f64.powf(zoom) * (256.0 / radius)).ceil();
        
        let mut grid: HashMap<(i32, i32), ClusterData> = HashMap::new();

        for point in &POINTS {
            // Filter by bounds (simple check)
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
        
        // Write results to buffer
        // Format: [lat, lng, count, id]
        for data in grid.values() {
            let count_f = data.count as f64;
            // Use average Lat/Lng for centroid
            let avg_lat = data.sum_lat / count_f;
            let avg_lng = data.sum_lng / count_f;
            
            OUTPUT_BUFFER.push(avg_lat);
            OUTPUT_BUFFER.push(avg_lng);
            OUTPUT_BUFFER.push(count_f);
            OUTPUT_BUFFER.push(data.first_id as f64);
        }
        
        OUTPUT_BUFFER.len() / 4
    }
}

// Simple buffer allocation for creating shared memory space if needed from JS
static mut BUFFER: [f64; 20000] = [0.0; 20000]; // Increased buffer size for points

#[no_mangle]
pub extern "C" fn get_buffer_ptr() -> *mut f64 {
    unsafe { BUFFER.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn get_output_buffer_ptr() -> *const f64 {
    unsafe { OUTPUT_BUFFER.as_ptr() }
}

#[no_mangle]
pub extern "C" fn calculate_bezier(
    x1: f64, y1: f64,
    x2: f64, y2: f64,
    curve_intensity: f64,
    segments: usize,
    ptr: *mut f64
) -> usize {
    // Safety: we assume the caller provides a pointer to a sufficiently large buffer.
    // Each point needs 2 f64s (x, y). Total size = (segments + 1) * 2 * 8 bytes.
    let buffer = unsafe { std::slice::from_raw_parts_mut(ptr, (segments + 1) * 2) };

    // Calculate midpoint
    let mid_x = (x1 + x2) / 2.0;
    let mid_y = (y1 + y2) / 2.0;

    // Calculate perpendicular offset for control point
    let dx = x2 - x1;
    let dy = y2 - y1;
    let distance = (dx * dx + dy * dy).sqrt();

    // Perpendicular direction (rotated 90 degrees)
    // Avoid division by zero
    let (perp_x, perp_y) = if distance > 0.0 {
        (-dy / distance, dx / distance)
    } else {
        (0.0, 0.0)
    };

    // Control point offset - alternate direction based on coordinates for variety
    // JavaScript code: const direction = (x1 + y1) % 2 === 0 ? 1 : -1
    // We approximate this behavior using integer casting
    let sum_coords = (x1 + y1) as i64;
    let direction = if sum_coords % 2 == 0 { 1.0 } else { -1.0 };
    let offset = distance * curve_intensity * direction;

    // Control point
    let cx = mid_x + perp_x * offset;
    let cy = mid_y + perp_y * offset;

    // Generate points along the quadratic Bezier curve
    for i in 0..=segments {
        let t = i as f64 / segments as f64;
        let one_minus_t = 1.0 - t;

        // Quadratic Bezier formula: B(t) = (1-t)²P0 + 2(1-t)tP1 + t²P2
        let x = one_minus_t * one_minus_t * x1 + 2.0 * one_minus_t * t * cx + t * t * x2;
        let y = one_minus_t * one_minus_t * y1 + 2.0 * one_minus_t * t * cy + t * t * y2;

        buffer[i * 2] = x;
        buffer[i * 2 + 1] = y;
    }

    // Return the number of points written
    segments + 1
}


// ... (previous clustering logic) ...

// --- Simulation Logic ---

#[derive(Clone, Copy)]
struct SimulationNode {
    node_type: u8, // 0=Gen, 1=Storage, 2=Consumer
    base_value: f64,
    current_value: f64,
    status: u8, // 0=Idle, 1=Active, 2=Maintenance
    is_real: u8, // 1=True, 0=False
}

#[derive(Clone, Copy)]
struct SimulationFlow {
    flow_index: u32,
    base_power: f64,
    current_power: f64,
}

static mut SIM_NODES: Vec<SimulationNode> = Vec::new();
static mut SIM_FLOWS: Vec<SimulationFlow> = Vec::new();
static mut SIM_NODE_OUTPUT: Vec<f64> = Vec::new(); // [val, status, val, status...]
static mut SIM_FLOW_OUTPUT: Vec<f64> = Vec::new(); // [val, val...]

// Simple LCG Random Number Generator
static mut MSG_RNG_STATE: u32 = 12345;
unsafe fn rand_float() -> f64 {
    MSG_RNG_STATE = MSG_RNG_STATE.wrapping_mul(1664525).wrapping_add(1013904223);
    (MSG_RNG_STATE as f64) / (u32::MAX as f64)
}

unsafe fn fluctuate(base_value: f64, percent_range: f64) -> f64 {
    let variance = base_value * (percent_range / 100.0);
    let rand = rand_float() * 2.0 - 1.0;
    base_value + rand * variance
}

fn get_time_multiplier(hour: f64, node_type: u8) -> f64 {
    let h = hour;
    if node_type == 0 { // Generator (Solar)
        if h >= 19.0 || h < 6.0 { return 0.05; }
        if h >= 6.0 && h < 8.0 { return 0.3; }
        if h >= 8.0 && h < 10.0 { return 0.6; }
        if h >= 10.0 && h < 12.0 { return 0.85; }
        if h >= 12.0 && h < 14.0 { return 1.0; }
        if h >= 14.0 && h < 16.0 { return 0.9; }
        if h >= 16.0 && h < 18.0 { return 0.6; }
        if h >= 18.0 && h < 19.0 { return 0.2; }
        return 0.5;
    } else if node_type == 2 { // Consumer
        if h >= 0.0 && h < 6.0 { return 0.2; }
        if h >= 6.0 && h < 8.0 { return 0.5; }
        if h >= 8.0 && h < 10.0 { return 0.9; }
        if h >= 10.0 && h < 12.0 { return 0.8; }
        if h >= 12.0 && h < 14.0 { return 0.6; } // Lunch
        if h >= 14.0 && h < 17.0 { return 0.85; }
        if h >= 17.0 && h < 20.0 { return 1.0; }
        if h >= 20.0 && h < 22.0 { return 0.7; }
        if h >= 22.0 { return 0.3; }
        return 0.5;
    } else if node_type == 1 { // Storage
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
    unsafe {
        SIM_NODES.clear();
        let input = std::slice::from_raw_parts(ptr, count * 5); // 5 fields per node
        
        for i in 0..count {
            let node_type = input[i * 5] as u8;
            let base_value = input[i * 5 + 1];
            let current_value = input[i * 5 + 2];
            let status = input[i * 5 + 3] as u8;
            let is_real = input[i * 5 + 4] as u8;
            
            SIM_NODES.push(SimulationNode {
                node_type,
                base_value,
                current_value,
                status,
                is_real,
            });
        }
    }
}

#[no_mangle]
pub extern "C" fn init_simulation_flows(ptr: *const f64, count: usize) {
    unsafe {
        SIM_FLOWS.clear();
        let input = std::slice::from_raw_parts(ptr, count * 2); // 2 fields (base, current)
        
        for i in 0..count {
            let base_power = input[i * 2];
            let current_power = input[i * 2 + 1];
            
            SIM_FLOWS.push(SimulationFlow {
                flow_index: i as u32,
                base_power,
                current_power,
            });
        }
    }
}

#[no_mangle]
pub extern "C" fn update_simulation(hour: f64, minute: f64) {
    unsafe {
        SIM_NODE_OUTPUT.clear();
        SIM_FLOW_OUTPUT.clear();
        
        let minute_variation = (minute / 60.0 * std::f64::consts::PI * 2.0).sin() * 0.05;
        
        // Update Nodes
        for node in SIM_NODES.iter_mut() {
            if node.is_real == 1 {
                // Real node, keep current value (could be updated via another API)
                // Just push to output
                 SIM_NODE_OUTPUT.push(node.current_value);
                 SIM_NODE_OUTPUT.push(node.status as f64);
                 continue;
            }
            
            let multiplier = get_time_multiplier(hour, node.node_type);
            let base_calculated = node.base_value * multiplier * (1.0 + minute_variation);
            
            // Fluctuate
            let new_value = fluctuate(base_calculated, 8.0).max(0.0);
            
            // Random status change (very rare)
            if rand_float() < 0.005 {
                 node.status = if rand_float() > 0.5 { 1 } else { 0 }; // Active/Idle
            }
            
            node.current_value = new_value;
            
            SIM_NODE_OUTPUT.push(new_value);
            SIM_NODE_OUTPUT.push(node.status as f64);
        }
        
        // Update Flows
        let gen_multiplier = get_time_multiplier(hour, 0); // Use generator schedule for flows
        for flow in SIM_FLOWS.iter_mut() {
             let base = flow.base_power * gen_multiplier;
             let new_power = fluctuate(base, 12.0).max(50.0);
             flow.current_power = new_power;
             SIM_FLOW_OUTPUT.push(new_power);
        }
    }
}

#[no_mangle]
pub extern "C" fn get_node_output_ptr() -> *const f64 {
    unsafe { SIM_NODE_OUTPUT.as_ptr() }
}

#[no_mangle]
pub extern "C" fn get_flow_output_ptr() -> *const f64 {
    unsafe { SIM_FLOW_OUTPUT.as_ptr() }
}
