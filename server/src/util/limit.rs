use crate::config::error::AppError;
use crate::config::state::IpRequestCountMap;
use axum::http::StatusCode;
use std::sync::LazyLock;

/// How many requests a single IP group may make per hour. Shared by the WebRTC
/// signaling and the relay endpoint.
static MAX_REQUESTS: LazyLock<u32> = LazyLock::new(|| {
    std::env::var("MAX_REQUESTS_PER_IP_PER_HOUR")
        .unwrap_or_else(|_| "1000".to_string())
        .parse::<u32>()
        .unwrap()
});

/// Increments the per-IP-request count, rejecting the request if the hourly
/// limit is reached (a cheap anti-DDoS guard).
pub(crate) async fn rate_limit(ip_group: &str, request_count_map: &IpRequestCountMap) -> Result<(), AppError> {
    let mut request_count_map = request_count_map.lock().await;
    let count = request_count_map.entry(ip_group.to_string()).or_insert(0);
    if *count >= *MAX_REQUESTS {
        return Err(AppError::status(StatusCode::TOO_MANY_REQUESTS, None));
    }
    *count += 1;
    Ok(())
}