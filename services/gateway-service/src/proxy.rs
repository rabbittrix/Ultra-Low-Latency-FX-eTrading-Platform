//! HTTP proxy handlers for forwarding requests to backend services

use axum::{
    body::Body,
    extract::{OriginalUri, Request},
    http::StatusCode,
    response::Response,
};
use std::env;
use tracing::{error, info};

/// Resolve backend base URL: full key `MATCHING_ENGINE_SERVICE_URL`, then legacy Compose
/// short key `MATCHING_ENGINE_URL` (service name with `-service` trimmed), then default host:port.
fn get_backend_url(service_name: &str, default_port: u16) -> String {
    let full_key = format!("{}_URL", service_name.to_uppercase().replace("-", "_"));
    if let Ok(url) = env::var(&full_key) {
        if !url.is_empty() {
            return url;
        }
    }
    let short = service_name.trim_end_matches("-service");
    if short != service_name {
        let legacy_key = format!("{}_URL", short.to_uppercase().replace("-", "_"));
        if let Ok(url) = env::var(&legacy_key) {
            if !url.is_empty() {
                return url;
            }
        }
    }
    // Docker Compose sets e.g. MATCHING_ENGINE_URL; bare `cargo run` expects localhost.
    format!("http://127.0.0.1:{}", default_port)
}

fn normalize_tail_path(path: &str) -> &str {
    path.trim_end_matches('/')
}

/// Tail path for the backend after `Router::nest("/{gw}", …)`.
///
/// Always derive from [`OriginalUri`] (client path). `Request::uri()` inside a nested service can
/// be the stripped remainder only (`/trades`), the full path (`/matching/trades`), or ambiguous;
/// using the original avoids forwarding `/matching/trades` to the backend as `matching/trades`.
fn nested_service_remainder<'a>(uri_path: &'a str, gateway_first_segment: &str) -> &'a str {
    let p = uri_path.trim_start_matches('/');
    let gw = gateway_first_segment.trim_matches('/');
    if p.len() > gw.len() && p.starts_with(gw) && p.as_bytes().get(gw.len()) == Some(&b'/') {
        return normalize_tail_path(&p[gw.len() + 1..]);
    }
    if p == gw {
        return "";
    }
    normalize_tail_path(p)
}

/// Proxy request to matching engine service
pub async fn proxy_matching(original_uri: OriginalUri, request: Request) -> Response {
    let base_url = get_backend_url("matching-engine-service", 8083);
    let path = nested_service_remainder(original_uri.path(), "matching");
    proxy_request(&base_url, path, request).await
}

/// Proxy request to risk service
pub async fn proxy_risk(original_uri: OriginalUri, request: Request) -> Response {
    let base_url = get_backend_url("risk-service", 8084);
    let path = nested_service_remainder(original_uri.path(), "risk");
    proxy_request(&base_url, path, request).await
}

/// Proxy request to market data service
pub async fn proxy_market_data(original_uri: OriginalUri, request: Request) -> Response {
    let base_url = get_backend_url("market-data-service", 8081);
    let path = nested_service_remainder(original_uri.path(), "market-data");
    proxy_request(&base_url, path, request).await
}

/// Proxy request to pricing service
pub async fn proxy_pricing(original_uri: OriginalUri, request: Request) -> Response {
    let base_url = get_backend_url("pricing-service", 8082);
    let path = nested_service_remainder(original_uri.path(), "pricing");
    proxy_request(&base_url, path, request).await
}

/// Global liquidity graph service (8091)
pub async fn proxy_liquidity(original_uri: OriginalUri, request: Request) -> Response {
    let base_url = get_backend_url("liquidity-graph-service", 8091);
    let path = nested_service_remainder(original_uri.path(), "liquidity");
    proxy_request(&base_url, path, request).await
}

/// AI-driven execution engine orchestrator (8092)
pub async fn proxy_execution(original_uri: OriginalUri, request: Request) -> Response {
    let base_url = get_backend_url("execution-engine", 8092);
    let path = nested_service_remainder(original_uri.path(), "execution");
    proxy_request(&base_url, path, request).await
}

/// Generic proxy function
async fn proxy_request(base_url: &str, path: &str, request: Request) -> Response {
    let method = request.method().clone();

    // Extract headers before consuming request
    let headers = request.headers().clone();

    // Build target URL
    let target_url = if path.is_empty() {
        base_url.to_string()
    } else if path.starts_with('/') {
        format!("{}{}", base_url, path)
    } else {
        format!("{}/{}", base_url, path)
    };

    info!("Proxying {} {} -> {}", method, path, target_url);

    // Extract body
    let body = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Failed to read request body"))
                .unwrap();
        }
    };

    // Build request to backend
    let client = reqwest::Client::new();
    let mut req_builder = client.request(method, &target_url);

    // Copy relevant headers from original request
    for (key, value) in headers.iter() {
        let key_str = key.as_str();
        // Skip headers that shouldn't be forwarded
        if key_str != "host"
            && key_str != "connection"
            && key_str != "transfer-encoding"
            && key_str != "content-length"
        {
            if let Ok(header_value) = value.to_str() {
                req_builder = req_builder.header(key, header_value);
            }
        }
    }

    // Set Content-Type if not already set and body is not empty
    if !body.is_empty() && !headers.contains_key("content-type") {
        req_builder = req_builder.header("Content-Type", "application/json");
    }

    req_builder = req_builder.body(body.to_vec());

    // Execute request
    match req_builder.send().await {
        Ok(response) => {
            let status = response.status();
            let response_headers = response.headers().clone();
            let body_bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("Failed to read response body: {}", e);
                    return Response::builder()
                        .status(StatusCode::BAD_GATEWAY)
                        .body(Body::from("Failed to read response body"))
                        .unwrap();
                }
            };

            let mut response_builder = Response::builder().status(status);

            // Copy response headers (excluding connection, transfer-encoding)
            for (key, value) in response_headers.iter() {
                if key.as_str() != "connection" && key.as_str() != "transfer-encoding" {
                    response_builder = response_builder.header(key, value);
                }
            }

            response_builder
                .body(Body::from(body_bytes.to_vec()))
                .unwrap_or_else(|e| {
                    error!("Failed to build response: {}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal server error"))
                        .unwrap()
                })
        }
        Err(e) => {
            error!("Proxy request failed: {}", e);
            let status = if e.is_timeout() {
                StatusCode::GATEWAY_TIMEOUT
            } else if e.is_connect() {
                StatusCode::BAD_GATEWAY
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Response::builder()
                .status(status)
                .body(Body::from(format!("Proxy error: {}", e)))
                .unwrap()
        }
    }
}
