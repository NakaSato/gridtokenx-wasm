//! Map Point Clustering Module
//! 
//! High-performance point clustering for map marker visualization.

use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::f64::consts::PI;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub lat: f64,
    pub lng: f64,
    pub id: u32,
}

#[derive(Clone, Copy)]
struct InternalPoint {
    x: f64, // Web Mercator X (0..1)
    y: f64, // Web Mercator Y (0..1)
    id: u32,
    lat: f64,
    lng: f64,
}

#[derive(Serialize)]
pub struct Cluster {
    pub lat: f64,
    pub lng: f64,
    pub count: u32,
    pub id: u32, // ID of first point in cluster (for representative icon)
}

#[wasm_bindgen]
pub struct Clusterer {
    points: Vec<InternalPoint>,
}

#[wasm_bindgen]
impl Clusterer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    pub fn load_points(&mut self, points: JsValue) -> Result<(), JsValue> {
        let input_points: Vec<Point> = serde_wasm_bindgen::from_value(points)?;
        self.points.clear();
        self.points.reserve(input_points.len());
        
        for p in input_points {
            self.points.push(InternalPoint {
                x: lng_to_x(p.lng),
                y: lat_to_y(p.lat),
                id: p.id,
                lat: p.lat,
                lng: p.lng,
            });
        }
        Ok(())
    }

    pub fn get_clusters(
        &self,
        min_lng: f64, min_lat: f64,
        max_lng: f64, max_lat: f64,
        zoom: f64
    ) -> Result<JsValue, JsValue> {
        let min_x = lng_to_x(min_lng);
        let max_x = lng_to_x(max_lng);
        let min_y = lat_to_y(max_lat);
        let max_y = lat_to_y(min_lat);

        let radius = 60.0;
        let cells = (2.0f64.powf(zoom) * (256.0 / radius)).ceil();
        
        let mut grid: HashMap<(i32, i32), ClusterData> = HashMap::new();

        for point in &self.points {
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

        let mut clusters = Vec::with_capacity(grid.len());
        for data in grid.values() {
            let count_f = data.count as f64;
            clusters.push(Cluster {
                lat: data.sum_lat / count_f,
                lng: data.sum_lng / count_f,
                count: data.count,
                id: data.first_id,
            });
        }
        
        Ok(serde_wasm_bindgen::to_value(&clusters)?)
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

// Web Mercator projection helpers
fn lng_to_x(lng: f64) -> f64 {
    (lng + 180.0) / 360.0
}

fn lat_to_y(lat: f64) -> f64 {
    let sin_lat = (lat * PI / 180.0).sin();
    let y = 0.5 - (0.25 * ((1.0 + sin_lat) / (1.0 - sin_lat)).ln() / PI);
    y.clamp(0.0, 1.0)
}
