use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ClusterCenter {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub color: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClusteringResult {
    pub user_position: Position,
    pub cluster_centers: Vec<ClusterCenter>,
    pub nearest_cluster: String,
    pub distance_to_cluster: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize)]
pub struct ProfileCharacteristics {
    pub peak_to_avg_ratio: f64,
    pub daytime_ratio: f64,
}

#[wasm_bindgen]
pub fn perform_clustering(chars: JsValue) -> Result<JsValue, JsValue> {
    let characteristics: ProfileCharacteristics = serde_wasm_bindgen::from_value(chars)?;
    
    // Normalize characteristics for clustering
    let user_x = (characteristics.peak_to_avg_ratio / 5.0).min(1.0);
    let user_y = characteristics.daytime_ratio;

    // Define cluster centers
    let cluster_centers = vec![
        ClusterCenter { id: "solar_enthusiast".to_string(), x: 0.6, y: 0.8, color: "#f59e0b".to_string(), name: "Solar Enthusiast".to_string() },
        ClusterCenter { id: "night_owl".to_string(), x: 0.5, y: 0.25, color: "#6366f1".to_string(), name: "Night Owl".to_string() },
        ClusterCenter { id: "energy_saver".to_string(), x: 0.2, y: 0.55, color: "#22c55e".to_string(), name: "Energy Saver".to_string() },
        ClusterCenter { id: "home_worker".to_string(), x: 0.35, y: 0.7, color: "#3b82f6".to_string(), name: "Home Worker".to_string() },
        ClusterCenter { id: "industrial".to_string(), x: 0.8, y: 0.6, color: "#ef4444".to_string(), name: "Industrial User".to_string() },
    ];

    let mut min_distance = f64::INFINITY;
    let mut nearest_cluster_id = cluster_centers[0].id.clone();

    for center in &cluster_centers {
        let distance = ((user_x - center.x).powi(2) + (user_y - center.y).powi(2)).sqrt();
        if distance < min_distance {
            min_distance = distance;
            nearest_cluster_id = center.id.clone();
        }
    }

    let result = ClusteringResult {
        user_position: Position { x: user_x, y: user_y },
        cluster_centers,
        nearest_cluster: nearest_cluster_id,
        distance_to_cluster: min_distance,
    };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}
