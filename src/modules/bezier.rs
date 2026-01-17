//! Bezier Curve Module
//! 
//! Quadratic Bezier curve generation for energy flow visualization.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn calculate_bezier(
    x1: f64, y1: f64,
    x2: f64, y2: f64,
    curve_intensity: f64,
    segments: usize,
) -> Vec<f64> {
    let mut buffer = Vec::with_capacity((segments + 1) * 2);

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

        // Quadratic Bezier formula
        let x = one_minus_t.powi(2) * x1 + 2.0 * one_minus_t * t * cx + t.powi(2) * x2;
        let y = one_minus_t.powi(2) * y1 + 2.0 * one_minus_t * t * cy + t.powi(2) * y2;

        buffer.push(x);
        buffer.push(y);
    }

    buffer
}
