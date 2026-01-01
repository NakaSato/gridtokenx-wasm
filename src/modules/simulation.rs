use std::f64::consts::PI;

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
        
        let minute_variation = (minute / 60.0 * PI * 2.0).sin() * 0.05;
        
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
