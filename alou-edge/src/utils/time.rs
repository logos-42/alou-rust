//! 时间工具函数 - 用于在WASM环境中安全地获取时间

/// 核心函数：获取当前时间戳（秒）- 统一返回 u64
pub fn current_timestamp_secs() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        use worker::js_sys::Date;
        (Date::now() / 1000.0) as u64
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// 核心函数：获取当前时间戳（毫秒）- 统一返回 u128
pub fn current_timestamp_millis() -> u128 {
    #[cfg(target_arch = "wasm32")]
    {
        use worker::js_sys::Date;
        Date::now() as u128
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }
}

// --- 以下函数基于核心函数进行类型转换，解决"打架"问题 ---

/// 获取当前时间戳（秒）- 返回 i64 用于兼容（解决 i64 冲突）
pub fn now_timestamp() -> i64 {
    current_timestamp_secs() as i64
}

/// 显式别名：获取当前时间戳（秒）返回 i64
pub fn now_timestamp_i64() -> i64 {
    current_timestamp_secs() as i64
}

/// 显式别名：获取当前时间戳（秒）返回 u64 (用于 TaskState)
pub fn current_timestamp_secs_u64() -> u64 {
    current_timestamp_secs()
}

/// 获取当前时间戳（毫秒）返回 i64 (解决 i64 冲突)
pub fn now_timestamp_millis_i64() -> i64 {
    current_timestamp_millis() as i64
}

/// 获取当前时间戳（毫秒）- 别名，用于向后兼容
pub fn now_timestamp_millis() -> u128 {
    current_timestamp_millis()
}

// --- 其他高精度及格式化工具 ---

/// 获取当前时间戳（秒，带小数部分）
pub fn current_timestamp_secs_f64() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        worker::js_sys::Date::now() / 1000.0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64()
    }
}

/// 获取当前时间戳（纳秒）- 用于加密等需要高精度时间的场景
pub fn now_timestamp_nanos() -> u128 {
    #[cfg(target_arch = "wasm32")]
    {
        (worker::js_sys::Date::now() as u128) * 1_000_000
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()
    }
}

/// 获取当前时间的 RFC3339 格式字符串
pub fn now_rfc3339() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        worker::js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Utc::now().to_rfc3339()
    }
}

/// 将时间戳转换为 RFC3339 格式字符串
pub fn timestamp_to_rfc3339(timestamp: u64) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let date = worker::js_sys::Date::new(&worker::js_sys::Number::from(timestamp as f64 * 1000.0));
        date.to_iso_string().as_string().unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use chrono::{DateTime, Utc};
        let d = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or(DateTime::<Utc>::MIN_UTC);
        d.to_rfc3339()
    }
}

/// 获取当前时间的格式化字符串（用于日志等）
pub fn now_formatted() -> String {
    now_rfc3339()
}