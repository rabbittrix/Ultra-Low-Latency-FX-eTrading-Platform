//! Ensures `nest("/matching", /*path)` forwards `/matching/audit` with inner path `/audit`.

use axum::{body::Body, extract::Request, http::StatusCode, routing::get, Router};
use tower::util::ServiceExt;

async fn echo_tail(request: Request) -> String {
    request
        .uri()
        .path()
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string()
}

fn matching_router() -> Router {
    Router::new().nest("/matching", Router::new().route("/*path", get(echo_tail)))
}

#[tokio::test]
async fn wildcard_audit_segment_is_captured() {
    let app = matching_router();
    let req = Request::builder()
        .uri("/matching/audit")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), b"audit");
}

#[tokio::test]
async fn options_matching_preflight_is_routed() {
    let app = Router::new().nest(
        "/matching",
        Router::new().route(
            "/*path",
            get(|| async { "ok" }).options(|| async { StatusCode::NO_CONTENT }),
        ),
    );
    let req = Request::builder()
        .method("OPTIONS")
        .uri("/matching/trades")
        .header("Origin", "http://localhost:3000")
        .header("Access-Control-Request-Method", "GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
