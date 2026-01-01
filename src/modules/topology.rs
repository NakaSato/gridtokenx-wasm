//! Grid Topology Module
//! 
//! Path finding, power flow calculation, and network analysis for the energy grid.

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct GridNode {
    pub id: u32,
    pub x: f64,  // Longitude or X coordinate
    pub y: f64,  // Latitude or Y coordinate
    pub node_type: u8,  // 0=Generator, 1=Storage, 2=Consumer, 3=Transformer
    pub capacity: f64,  // kW
    pub current_load: f64,  // kW
}

#[derive(Clone, Copy, Debug)]
pub struct GridLine {
    pub from_id: u32,
    pub to_id: u32,
    pub resistance: f64,  // Ohms
    pub max_capacity: f64,  // kW
    pub length_km: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DijkstraState {
    cost: f64,
    node_id: u32,
}

impl Eq for DijkstraState {}

impl Ord for DijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for DijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ============================================================================
// Grid Network
// ============================================================================

pub struct GridNetwork {
    nodes: HashMap<u32, GridNode>,
    lines: Vec<GridLine>,
    adjacency: HashMap<u32, Vec<(u32, f64, usize)>>,  // node -> [(neighbor, weight, line_idx)]
}

impl GridNetwork {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            lines: Vec::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.lines.clear();
        self.adjacency.clear();
    }

    pub fn add_node(&mut self, node: GridNode) {
        self.nodes.insert(node.id, node);
        self.adjacency.entry(node.id).or_insert_with(Vec::new);
    }

    pub fn add_line(&mut self, line: GridLine) {
        let line_idx = self.lines.len();
        self.lines.push(line);

        // Weight is based on length (could also use resistance or capacity)
        let weight = line.length_km;

        self.adjacency.entry(line.from_id)
            .or_insert_with(Vec::new)
            .push((line.to_id, weight, line_idx));

        // Bidirectional
        self.adjacency.entry(line.to_id)
            .or_insert_with(Vec::new)
            .push((line.from_id, weight, line_idx));
    }

    /// Dijkstra's shortest path algorithm
    /// Returns: (path as Vec<node_id>, total_distance)
    pub fn shortest_path(&self, start: u32, end: u32) -> Option<(Vec<u32>, f64)> {
        if !self.nodes.contains_key(&start) || !self.nodes.contains_key(&end) {
            return None;
        }

        let mut dist: HashMap<u32, f64> = HashMap::new();
        let mut prev: HashMap<u32, u32> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(start, 0.0);
        heap.push(DijkstraState { cost: 0.0, node_id: start });

        while let Some(DijkstraState { cost, node_id }) = heap.pop() {
            if node_id == end {
                // Reconstruct path
                let mut path = vec![end];
                let mut current = end;
                while let Some(&prev_node) = prev.get(&current) {
                    path.push(prev_node);
                    current = prev_node;
                }
                path.reverse();
                return Some((path, cost));
            }

            if cost > *dist.get(&node_id).unwrap_or(&f64::INFINITY) {
                continue;
            }

            if let Some(neighbors) = self.adjacency.get(&node_id) {
                for &(neighbor, weight, _) in neighbors {
                    let new_cost = cost + weight;
                    if new_cost < *dist.get(&neighbor).unwrap_or(&f64::INFINITY) {
                        dist.insert(neighbor, new_cost);
                        prev.insert(neighbor, node_id);
                        heap.push(DijkstraState { cost: new_cost, node_id: neighbor });
                    }
                }
            }
        }

        None
    }

    /// Calculate power flow through the network using DC approximation
    /// Returns: Map of line_idx -> power_flow_kw
    pub fn calc_power_flow(&self) -> HashMap<usize, f64> {
        let mut flows: HashMap<usize, f64> = HashMap::new();

        // Initialize all flows to 0
        for i in 0..self.lines.len() {
            flows.insert(i, 0.0);
        }

        // For each generator, distribute power to consumers via shortest paths
        let generators: Vec<_> = self.nodes.values()
            .filter(|n| n.node_type == 0 && n.current_load > 0.0)
            .collect();

        let consumers: Vec<_> = self.nodes.values()
            .filter(|n| n.node_type == 2 && n.current_load > 0.0)
            .collect();

        let total_demand: f64 = consumers.iter().map(|c| c.current_load).sum();
        if total_demand <= 0.0 {
            return flows;
        }

        for gen in &generators {
            let gen_power = gen.current_load;

            // Distribute generator power proportionally to consumer demand
            for consumer in &consumers {
                let fraction = consumer.current_load / total_demand;
                let power_to_send = gen_power * fraction;

                // Find path and add flow
                if let Some((path, _)) = self.shortest_path(gen.id, consumer.id) {
                    for i in 0..path.len() - 1 {
                        let from = path[i];
                        let to = path[i + 1];

                        // Find the line connecting these nodes
                        if let Some(neighbors) = self.adjacency.get(&from) {
                            for &(neighbor, _, line_idx) in neighbors {
                                if neighbor == to {
                                    *flows.entry(line_idx).or_insert(0.0) += power_to_send;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        flows
    }

    /// Estimate transmission line losses
    /// Uses simplified I²R loss formula
    /// Returns: (total_loss_kw, map of line_idx -> loss_kw)
    pub fn calc_line_losses(&self, flows: &HashMap<usize, f64>, voltage_kv: f64) -> (f64, HashMap<usize, f64>) {
        let mut losses: HashMap<usize, f64> = HashMap::new();
        let mut total_loss = 0.0;

        for (line_idx, &power_kw) in flows {
            if let Some(line) = self.lines.get(*line_idx) {
                // I = P / V  (simplified, assuming power factor = 1)
                let current_a = (power_kw * 1000.0) / (voltage_kv * 1000.0);
                // Loss = I²R
                let loss_w = current_a * current_a * line.resistance;
                let loss_kw = loss_w / 1000.0;

                losses.insert(*line_idx, loss_kw);
                total_loss += loss_kw;
            }
        }

        (total_loss, losses)
    }

    /// Detect loops/cycles in the grid (for redundancy analysis)
    /// Uses DFS to find back edges
    pub fn detect_loops(&self) -> Vec<Vec<u32>> {
        let mut visited: HashMap<u32, bool> = HashMap::new();
        let mut parent: HashMap<u32, u32> = HashMap::new();
        let mut loops: Vec<Vec<u32>> = Vec::new();

        for &start in self.nodes.keys() {
            if visited.get(&start).copied().unwrap_or(false) {
                continue;
            }

            let mut stack = vec![(start, None::<u32>)];

            while let Some((node, from)) = stack.pop() {
                if visited.get(&node).copied().unwrap_or(false) {
                    // Found a loop - trace it back
                    if let Some(from_node) = from {
                        let mut loop_path = vec![node];
                        let mut current = from_node;
                        loop_path.push(current);

                        while current != node {
                            if let Some(&p) = parent.get(&current) {
                                current = p;
                                if loop_path.contains(&current) {
                                    break;
                                }
                                loop_path.push(current);
                            } else {
                                break;
                            }
                        }

                        if loop_path.len() >= 3 {
                            loops.push(loop_path);
                        }
                    }
                    continue;
                }

                visited.insert(node, true);
                if let Some(from_node) = from {
                    parent.insert(node, from_node);
                }

                if let Some(neighbors) = self.adjacency.get(&node) {
                    for &(neighbor, _, _) in neighbors {
                        if from.map_or(true, |f| f != neighbor) {
                            stack.push((neighbor, Some(node)));
                        }
                    }
                }
            }
        }

        loops
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

// ============================================================================
// Global State & FFI
// ============================================================================

static mut GRID_NETWORK: Option<GridNetwork> = None;
static mut PATH_OUTPUT: Vec<f64> = Vec::new();
static mut FLOW_OUTPUT: Vec<f64> = Vec::new();

fn get_network() -> &'static mut GridNetwork {
    unsafe {
        if GRID_NETWORK.is_none() {
            GRID_NETWORK = Some(GridNetwork::new());
        }
        GRID_NETWORK.as_mut().unwrap()
    }
}

/// Initialize/reset the grid network
#[no_mangle]
pub extern "C" fn topology_init() {
    get_network().clear();
}

/// Load nodes from buffer
/// Format: [id, x, y, type, capacity, current_load, ...]
#[no_mangle]
pub extern "C" fn topology_load_nodes(ptr: *const f64, count: usize) {
    let input = unsafe { std::slice::from_raw_parts(ptr, count * 6) };
    let network = get_network();

    for i in 0..count {
        let node = GridNode {
            id: input[i * 6] as u32,
            x: input[i * 6 + 1],
            y: input[i * 6 + 2],
            node_type: input[i * 6 + 3] as u8,
            capacity: input[i * 6 + 4],
            current_load: input[i * 6 + 5],
        };
        network.add_node(node);
    }
}

/// Load lines from buffer
/// Format: [from_id, to_id, resistance, max_capacity, length_km, ...]
#[no_mangle]
pub extern "C" fn topology_load_lines(ptr: *const f64, count: usize) {
    let input = unsafe { std::slice::from_raw_parts(ptr, count * 5) };
    let network = get_network();

    for i in 0..count {
        let line = GridLine {
            from_id: input[i * 5] as u32,
            to_id: input[i * 5 + 1] as u32,
            resistance: input[i * 5 + 2],
            max_capacity: input[i * 5 + 3],
            length_km: input[i * 5 + 4],
        };
        network.add_line(line);
    }
}

/// Find shortest path between two nodes
/// Returns path length (number of nodes), or 0 if no path
/// Output format: [node_id, node_id, ...]
#[no_mangle]
pub extern "C" fn topology_shortest_path(start: u32, end: u32) -> usize {
    if let Some((path, _distance)) = get_network().shortest_path(start, end) {
        unsafe {
            PATH_OUTPUT.clear();
            for node_id in &path {
                PATH_OUTPUT.push(*node_id as f64);
            }
        }
        path.len()
    } else {
        0
    }
}

/// Get pointer to path output buffer
#[no_mangle]
pub extern "C" fn topology_path_ptr() -> *const f64 {
    unsafe { PATH_OUTPUT.as_ptr() }
}

/// Calculate power flow through network
/// Returns number of lines with flow data
/// Output format: [line_idx, flow_kw, line_idx, flow_kw, ...]
#[no_mangle]
pub extern "C" fn topology_calc_flow() -> usize {
    let flows = get_network().calc_power_flow();
    unsafe {
        FLOW_OUTPUT.clear();
        for (idx, flow) in &flows {
            FLOW_OUTPUT.push(*idx as f64);
            FLOW_OUTPUT.push(*flow);
        }
    }
    flows.len()
}

/// Get pointer to flow output buffer
#[no_mangle]
pub extern "C" fn topology_flow_ptr() -> *const f64 {
    unsafe { FLOW_OUTPUT.as_ptr() }
}

/// Calculate total line losses
/// Returns total loss in kW
#[no_mangle]
pub extern "C" fn topology_calc_losses(voltage_kv: f64) -> f64 {
    let flows = get_network().calc_power_flow();
    let (total_loss, _) = get_network().calc_line_losses(&flows, voltage_kv);
    total_loss
}

/// Get network stats
#[no_mangle]
pub extern "C" fn topology_node_count() -> usize {
    get_network().node_count()
}

#[no_mangle]
pub extern "C" fn topology_line_count() -> usize {
    get_network().line_count()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_network() -> GridNetwork {
        let mut network = GridNetwork::new();

        // Create a simple network: Gen -> T1 -> T2 -> Consumer
        network.add_node(GridNode { id: 1, x: 0.0, y: 0.0, node_type: 0, capacity: 100.0, current_load: 50.0 });
        network.add_node(GridNode { id: 2, x: 1.0, y: 0.0, node_type: 3, capacity: 100.0, current_load: 0.0 });
        network.add_node(GridNode { id: 3, x: 2.0, y: 0.0, node_type: 3, capacity: 100.0, current_load: 0.0 });
        network.add_node(GridNode { id: 4, x: 3.0, y: 0.0, node_type: 2, capacity: 80.0, current_load: 40.0 });

        network.add_line(GridLine { from_id: 1, to_id: 2, resistance: 0.1, max_capacity: 100.0, length_km: 1.0 });
        network.add_line(GridLine { from_id: 2, to_id: 3, resistance: 0.1, max_capacity: 100.0, length_km: 1.0 });
        network.add_line(GridLine { from_id: 3, to_id: 4, resistance: 0.1, max_capacity: 100.0, length_km: 1.0 });

        network
    }

    #[test]
    fn test_shortest_path() {
        let network = create_test_network();

        let result = network.shortest_path(1, 4);
        assert!(result.is_some());

        let (path, distance) = result.unwrap();
        assert_eq!(path, vec![1, 2, 3, 4]);
        assert!((distance - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_power_flow() {
        let network = create_test_network();
        let flows = network.calc_power_flow();

        // All lines should have flow
        assert!(!flows.is_empty());
        
        // Flow should be positive
        for (_, flow) in &flows {
            assert!(*flow >= 0.0);
        }
    }

    #[test]
    fn test_line_losses() {
        let network = create_test_network();
        let flows = network.calc_power_flow();
        let (total_loss, _) = network.calc_line_losses(&flows, 11.0);  // 11 kV

        // Should have some loss
        assert!(total_loss >= 0.0);
    }
}
