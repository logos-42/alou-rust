/// Time utilities for WASM environment
/// 
/// Standard library time functions don't work in WASM, so we use js-sys::Date

use js_sys::Date;

/// Get current timestamp in seconds (Unix timestamp)
pub fn now_timestamp() -> i64 {
    (Date::now() / 1000.0) as i64
}

/// Get current timestamp in milliseconds
pub fn now_timestamp_millis() -> i64 {
    Date::now() as i64
}

/// Get current timestamp in nanoseconds (approximated from milliseconds)
pub fn now_timestamp_nanos() -> i64 {
    Date::now() as i64 * 1_000_000
}

/// Get current time as RFC3339 string
pub fn now_rfc3339() -> String {
    let date = Date::new_0();
    date.to_iso_string().as_string().unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

/// Format timestamp as RFC3339 string
#[allow(dead_code)]
pub fn timestamp_to_rfc3339(timestamp: i64) -> String {
    let date = Date::new(&(timestamp * 1000).into());
    date.to_iso_string().as_string().unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

/// Format current time as YYYY-MM-DD HH:MM:SS
pub fn now_formatted() -> String {
    let date = Date::new_0();
    let iso = date.to_iso_string().as_string().unwrap_or_default();
    // Convert ISO format to YYYY-MM-DD HH:MM:SS
    if iso.len() >= 19 {
        format!("{} {}", &iso[0..10], &iso[11..19])
    } else {
        "1970-01-01 00:00:00".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_now_timestamp() {
        let ts = now_timestamp();
        assert!(ts > 1700000000); // After 2023
    }
    
    #[test]
    fn test_now_rfc3339() {
        let rfc = now_rfc3339();
        assert!(rfc.contains("T"));
        assert!(rfc.contains("Z") || rfc.contains("+"));
    }
}
