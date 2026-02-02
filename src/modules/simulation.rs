//! Energy Flow Simulation Module
//! 
//! Time-based simulation of energy nodes and power flows.

use wasm_bindgen::prelude::*;
use std::f64::consts::PI;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct SimulationNode {
    #[serde(rename = "type", alias = "node_type")]
    pub node_type: u8,
    #[serde(rename = "base", alias = "base_value")]
    pub base_value: f64,
    #[serde(rename = "current", alias = "current_value")]
    pub current_value: f64,
    pub status: u8,
    #[serde(rename = "isReal", alias = "is_real")]
    pub is_real: u8,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct SimulationFlow {
    #[serde(rename = "index", alias = "flow_index")]
    pub flow_index: u32,
    #[serde(rename = "base", alias = "base_power")]
    pub base_power: f64,
    #[serde(rename = "current", alias = "current_power")]
    pub current_power: f64,
}

#[wasm_bindgen]
pub struct Simulation {
    nodes: Vec<SimulationNode>,
    flows: Vec<SimulationFlow>,
    rng_state: u32,
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            flows: Vec::new(),
            rng_state: 12345,
        }
    }





    pub fn set_nodes(&mut self, nodes: JsValue) -> Result<(), JsValue> {
        let nodes_vec: Vec<SimulationNode> = serde_wasm_bindgen::from_value(nodes)?;
        self.nodes = nodes_vec;
        Ok(())
    }

    pub fn set_flows(&mut self, flows: JsValue) -> Result<(), JsValue> {
        let flows_vec: Vec<SimulationFlow> = serde_wasm_bindgen::from_value(flows)?;
        self.flows = flows_vec;
        Ok(())
    }

    pub fn update(&mut self, hour: f64, minute: f64) {
        let minute_variation = (minute / 60.0 * PI * 2.0).sin() * 0.05;
        let mut rng_state = self.rng_state;

        // Use a closure or helper function for random numbers to avoid borrowing self
        let mut rand_float = || {
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            (rng_state as f64) / (u32::MAX as f64)
        };

        let fluctuate = |base_value: f64, percent_range: f64, rng: &mut dyn FnMut() -> f64| {
            let variance = base_value * (percent_range / 100.0);
            let rand = rng() * 2.0 - 1.0;
            base_value + rand * variance
        };

        for node in self.nodes.iter_mut() {
            if node.is_real == 1 {
                continue;
            }

            let multiplier = get_time_multiplier(hour, node.node_type);
            let base_calculated = node.base_value * multiplier * (1.0 + minute_variation);
            let new_value = fluctuate(base_calculated, 8.0, &mut rand_float).max(0.0);

            if rand_float() < 0.005 {
                node.status = if rand_float() > 0.5 { 1 } else { 0 };
            }

            node.current_value = new_value;
        }

        let gen_multiplier = get_time_multiplier(hour, 0);
        for flow in self.flows.iter_mut() {
            let base = flow.base_power * gen_multiplier;
            let new_power = fluctuate(base, 12.0, &mut rand_float).max(50.0);
            flow.current_power = new_power;
        }
        
        self.rng_state = rng_state;
    }

    /// Returns the current state of all nodes
    pub fn get_nodes(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.nodes)?)
    }

    /// Returns the current state of all flows
    pub fn get_flows(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.flows)?)
    }
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
