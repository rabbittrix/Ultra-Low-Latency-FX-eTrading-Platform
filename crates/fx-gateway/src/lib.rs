//! API Gateway for aggregating all services

pub mod api;
pub mod handlers;
mod openapi_proxy;
mod proxy_types;

pub use api::GatewayApi;
pub use handlers::{health, root};
