use axum::http::HeaderMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;

pub(crate) fn get_ip_group(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(ip) => {
            // IPv4: Each IP address is already a group.
            ip.to_string()
        }
        IpAddr::V6(ip) => {
            // IPv6: We treat /64 as a group.
            let segments = ip.segments();
            format!(
                "{:x}:{:x}:{:x}:{:x}",
                segments[0], segments[1], segments[2], segments[3]
            )
        }
    }
}

/// The real client IP: prefer the `x-forwarded-for` value (the last hop, which
/// a trusted proxy appended), falling back to the socket address.
pub(crate) fn client_ip(headers: &HeaderMap, addr: &SocketAddr) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').last()) // Get the last component
        .map(|v| v.trim().to_string())
        .and_then(|v| IpAddr::from_str(&v).ok())
        .unwrap_or(addr.ip())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_get_ip_group() {
        assert_eq!(
            get_ip_group(IpAddr::from_str("1.2.3.4").unwrap()),
            "1.2.3.4"
        );
        assert_eq!(
            get_ip_group(IpAddr::from_str("1:2:3:4:5:6:7:8").unwrap()),
            "1:2:3:4"
        );
        assert_eq!(
            get_ip_group(IpAddr::from_str("a:b:c:d:e:f:0:1").unwrap()),
            "a:b:c:d"
        );
    }
}
