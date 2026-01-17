//! Network Topology Analysis Module
//! 
//! Graph algorithms for grid network analysis (pathfinding, redundancy).

use wasm_bindgen::prelude::*;
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub node_type: String, // "source", "substation", "consumer"
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub capacity: f64,
    pub load: f64,
}

#[derive(Serialize)]
pub struct PathResult {
    pub path: Vec<String>,
    pub total_distance: f64,
}

#[wasm_bindgen]
pub struct Topology {
    nodes: HashMap<String, Node>,
    adj: HashMap<String, Vec<(String, f64)>>, // Adjacency list (target, distance)
}

#[wasm_bindgen]
impl Topology {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            adj: HashMap::new(),
        }
    }

    pub fn set_graph(&mut self, nodes: JsValue, edges: JsValue) -> Result<(), JsValue> {
        let nodes_vec: Vec<Node> = serde_wasm_bindgen::from_value(nodes)?;
        let edges_vec: Vec<Edge> = serde_wasm_bindgen::from_value(edges)?;
        
        self.nodes.clear();
        self.adj.clear();
        
        for node in nodes_vec {
            self.nodes.insert(node.id.clone(), node);
        }
        
        for edge in edges_vec {
            // Calculate Euclidean distance as weight
            let n1 = self.nodes.get(&edge.from).ok_or("Node not found")?;
            let n2 = self.nodes.get(&edge.to).ok_or("Node not found")?;
            let dist = ((n1.x - n2.x).powi(2) + (n1.y - n2.y).powi(2)).sqrt();
            
            self.adj.entry(edge.from.clone()).or_default().push((edge.to.clone(), dist));
            self.adj.entry(edge.to.clone()).or_default().push((edge.from.clone(), dist));
        }
        
        Ok(())
    }

    pub fn find_path(&self, start_id: &str, end_id: &str) -> Result<JsValue, JsValue> {
        let mut dist: HashMap<String, f64> = HashMap::new();
        let mut prev: HashMap<String, String> = HashMap::new();
        let mut heap = BinaryHeap::new();
        
        dist.insert(start_id.to_string(), 0.0);
        heap.push(State { cost: 0.0, position: start_id.to_string() });
        
        while let Some(State { cost, position }) = heap.pop() {
            if position == end_id {
                // Reconstruct path
                let mut path = Vec::new();
                let mut curr = end_id.to_string();
                while curr != start_id {
                    path.push(curr.clone());
                    curr = prev.get(&curr).cloned().unwrap();
                }
                path.push(start_id.to_string());
                path.reverse();
                
                return Ok(serde_wasm_bindgen::to_value(&PathResult {
                    path,
                    total_distance: cost,
                })?);
            }
            
            if cost > *dist.get(&position).unwrap_or(&f64::INFINITY) {
                continue;
            }
            
            if let Some(neighbors) = self.adj.get(&position) {
                for (neighbor, d) in neighbors {
                    let next_cost = cost + d;
                    if next_cost < *dist.get(neighbor).unwrap_or(&f64::INFINITY) {
                        heap.push(State { cost: next_cost, position: neighbor.clone() });
                        dist.insert(neighbor.clone(), next_cost);
                        prev.insert(neighbor.clone(), position.clone());
                    }
                }
            }
        }
        
        Ok(JsValue::NULL)
    }

    pub fn find_critical_nodes(&self) -> Result<JsValue, JsValue> {
        // Simple articulation point finding (simplified for brevity)
        // Just return substations for now as "critical"
        let mut critical = Vec::new();
        for node in self.nodes.values() {
            if node.node_type == "substation" {
                critical.push(node.id.clone());
            }
        }
        Ok(serde_wasm_bindgen::to_value(&critical)?)
    }
}

#[derive(PartialEq)]
struct State {
    cost: f64,
    position: String,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
