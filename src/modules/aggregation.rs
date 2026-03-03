use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct HourlyDataPoint {
    pub hour: String,
    pub consumption: f64,
    pub is_daytime: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AggregationResult {
    pub hourly_distribution: Vec<HourlyDataPoint>,
    pub total_kwh: f64,
    pub daytime_consumption: f64,
    pub nighttime_consumption: f64,
}

#[derive(Serialize, Deserialize)]
pub struct RawReading {
    pub timestamp: Option<String>,
    pub reading_timestamp: Option<String>,
    pub created_at: Option<String>,
    pub kwh: Option<String>,
    pub kwh_amount: Option<String>,
}

#[wasm_bindgen]
pub fn aggregate_readings(readings_js: JsValue) -> Result<JsValue, JsValue> {
    let readings: Vec<RawReading> = serde_wasm_bindgen::from_value(readings_js)?;
    
    let mut hourly_buckets: Vec<Vec<f64>> = vec![vec![]; 24];
    let mut total_kwh = 0.0;

    for r in readings {
        let kwh_str = r.kwh.or(r.kwh_amount).unwrap_or_else(|| "0".to_string());
        let kwh = kwh_str.parse::<f64>().unwrap_or(0.0);
        total_kwh += kwh;

        // Simplified hour extraction for WASM/Rust (assuming ISO string or similar)
        // In a real scenario, we'd use a chrono-like parser or pass timestamps as integers.
        // For compatibility with the current hook's "new Date(s).getHours()" approach:
        let timestamp = r.timestamp.or(r.reading_timestamp).or(r.created_at);
        if let Some(ts) = timestamp {
            // Very basic hour extraction from "2026-02-28T08:45:19" or similar
            if let Some(hour_part) = ts.split('T').nth(1).and_then(|t| t.split(':').next()) {
                if let Ok(hour) = hour_part.parse::<usize>() {
                    if hour < 24 {
                        hourly_buckets[hour].push(kwh);
                    }
                }
            }
        }
    }

    let mut hourly_distribution = Vec::with_capacity(24);
    let mut daytime_consumption = 0.0;
    let mut nighttime_consumption = 0.0;

    for hour in 0..24 {
        let bucket = &hourly_buckets[hour];
        let avg_kwh = if !bucket.is_empty() {
            bucket.iter().sum::<f64>() / bucket.len() as f64
        } else {
            0.0 // Hook has fallback logic, but WASM should be deterministic
        };

        let is_daytime = hour >= 6 && hour < 18;
        if is_daytime {
            daytime_consumption += avg_kwh;
        } else {
            nighttime_consumption += avg_kwh;
        }

        hourly_distribution.push(HourlyDataPoint {
            hour: format!("{:02}:00", hour),
            consumption: avg_kwh,
            is_daytime,
        });
    }

    let result = AggregationResult {
        hourly_distribution,
        total_kwh,
        daytime_consumption,
        nighttime_consumption,
    };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}
