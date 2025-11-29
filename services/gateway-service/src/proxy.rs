//! HTTP proxy handlers for forwarding requests to backend services

use axum::{
    body::Body,
    extract::{Path, Request},
    http::StatusCode,
    response::Response,
};
use std::env;
use tracing::{error, info};

/// Get backend service URL from environment variable
fn get_backend_url(service_name: &str, default_port: u16) -> String {
    let env_var = format!("{}_URL", service_name.to_uppercase().replace("-", "_"));
    env::var(&env_var).unwrap_or_else(|_| {
        let service_host = service_name.replace("-", "-");
        format!("http://{}:{}", service_host, default_port)
    })
}

/// Proxy request to matching engine service
/// Strips "/matching" prefix from path
pub async fn proxy_matching(Path(path): Path<String>, request: Request) -> Response {
    let base_url = get_backend_url("matching-engine-service", 8083);
    // Path already has the matching prefix stripped by the route
    proxy_request(&base_url, &path, request).await
}

/// Proxy request to risk service
/// Strips "/risk" prefix from path
pub async fn proxy_risk(Path(path): Path<String>, request: Request) -> Response {
    let base_url = get_backend_url("risk-service", 8084);
    // Path already has the risk prefix stripped by the route
    proxy_request(&base_url, &path, request).await
}

/// Proxy request to market data service
pub async fn proxy_market_data(Path(path): Path<String>, request: Request) -> Response {
    let base_url = get_backend_url("market-data-service", 8081);
    proxy_request(&base_url, &path, request).await
}

/// Proxy request to pricing service
pub async fn proxy_pricing(Path(path): Path<String>, request: Request) -> Response {
    let base_url = get_backend_url("pricing-service", 8082);
    proxy_request(&base_url, &path, request).await
}

/// Generic proxy function
async fn proxy_request(base_url: &str, path: &str, request: Request) -> Response {
    let method = request.method().clone();

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

    // Copy headers (excluding host and connection)
    // Note: In a real implementation, you'd want to filter headers more carefully
    req_builder = req_builder.body(body.to_vec());

    // Execute request
    match req_builder.send().await {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
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
            for (key, value) in headers.iter() {
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
