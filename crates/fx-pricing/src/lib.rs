//! Pricing engine for generating spreads and risk-adjusted prices

pub mod engine;
pub mod spread;

pub use engine::PricingEngine;
pub use spread::SpreadModel;
