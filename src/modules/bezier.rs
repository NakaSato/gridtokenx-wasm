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
