use actix_web::HttpRequest;
use std::net::IpAddr;

pub fn client_ip(req: &HttpRequest) -> String {
    let trusted: Vec<IpAddr> = std::env::var("TRUSTED_PROXY_IPS")
        .unwrap_or_default()
        .split(',')
        .filter_map(|ip| ip.trim().parse().ok())
        .collect();
    resolve(req, &trusted)
}

fn resolve(req: &HttpRequest, trusted: &[IpAddr]) -> String {
    let peer = req.peer_addr().map(|address| address.ip());
    if peer.is_some_and(|ip| trusted.contains(&ip)) {
        // Walk from the nearest proxy; never accept an arbitrary leftmost IP.
        if let Some(chain) = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
        {
            for raw in chain.rsplit(',') {
                let Ok(ip) = raw.trim().parse::<IpAddr>() else {
                    return peer.unwrap().to_string();
                };
                if !trusted.contains(&ip) {
                    return ip.to_string();
                }
            }
        }
    }
    peer.map(|ip| ip.to_string())
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn auth_forwarded_ip_requires_a_trusted_peer() {
        let req = TestRequest::default()
            .peer_addr("127.0.0.1:1234".parse().unwrap())
            .insert_header(("x-forwarded-for", "192.0.2.1, 198.51.100.2"))
            .to_http_request();
        assert_eq!(resolve(&req, &[]), "127.0.0.1");
        assert_eq!(
            resolve(&req, &["127.0.0.1".parse().unwrap()]),
            "198.51.100.2"
        );
        let bad = TestRequest::default()
            .peer_addr("127.0.0.1:1234".parse().unwrap())
            .insert_header(("x-forwarded-for", "invalid-ip"))
            .to_http_request();
        assert_eq!(resolve(&bad, &["127.0.0.1".parse().unwrap()]), "127.0.0.1");
    }
}
