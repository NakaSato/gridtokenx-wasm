use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64, // Web Mercator X (0..1)
    pub y: f64, // Web Mercator Y (0..1)
    pub id: u32,
    pub lat: f64,
    pub lng: f64,
}

static mut POINTS: Vec<Point> = Vec::new();
static mut OUTPUT_BUFFER: Vec<f64> = Vec::new();

// Web Mercator projection helpers
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

#[no_mangle]
pub extern "C" fn get_output_buffer_ptr() -> *const f64 {
    unsafe { OUTPUT_BUFFER.as_ptr() }
}
