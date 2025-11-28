//! API Gateway for aggregating all services

pub mod api;
pub mod handlers;

pub use api::GatewayApi;
pub use handlers::{health, root};
