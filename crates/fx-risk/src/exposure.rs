//! Exposure calculation and aggregation

use crate::engine::Position;
use crate::limits::RiskLimits;
use dashmap::DashMap;
use fx_utils::{OrderId, Quantity};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Exposure metrics for an instrument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentExposure {
    pub instrument: String,
    pub position: i64,
    pub position_abs: u64,
    pub position_utilization: f64, // Percentage of limit used
    pub open_orders_count: usize,
}

/// Overall exposure summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureSummary {
    pub total_instruments: usize,
    pub total_open_orders: usize,
    pub total_exposure: u64, // Sum of absolute positions
    pub instruments: Vec<InstrumentExposure>,
    pub risk_limits: RiskLimitsInfo,
}

/// Risk limits information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimitsInfo {
    pub max_position_size: u64,
    pub max_order_size: u64,
    pub max_daily_loss: u64,
    pub max_open_orders: usize,
}

impl From<&RiskLimits> for RiskLimitsInfo {
    fn from(limits: &RiskLimits) -> Self {
        Self {
            max_position_size: limits.max_position_size.0,
            max_order_size: limits.max_order_size.0,
            max_daily_loss: limits.max_daily_loss,
            max_open_orders: limits.max_open_orders,
        }
    }
}

/// Exposure calculator
pub struct ExposureCalculator;

impl ExposureCalculator {
    /// Calculate exposure for all instruments
    pub fn calculate_exposure(
        positions: &DashMap<String, Position>,
        open_orders: &DashMap<OrderId, Quantity>,
        limits: &Arc<RiskLimits>,
    ) -> ExposureSummary {
        let mut instruments = Vec::new();
        let mut total_exposure = 0u64;

        for entry in positions.iter() {
            let position = entry.value();
            let position_abs = position.quantity.unsigned_abs();
            total_exposure += position_abs;

            let position_utilization = if limits.max_position_size.0 > 0 {
                (position_abs as f64 / limits.max_position_size.0 as f64) * 100.0
            } else {
                0.0
            };

            // Count open orders for this instrument (simplified - in real implementation,
            // we'd track which orders belong to which instrument)
            let open_orders_count = open_orders.len();

            instruments.push(InstrumentExposure {
                instrument: position.instrument.clone(),
                position: position.quantity,
                position_abs,
                position_utilization,
                open_orders_count,
            });
        }

        // Sort by exposure (highest first)
        instruments.sort_by(|a, b| b.position_abs.cmp(&a.position_abs));

        ExposureSummary {
            total_instruments: instruments.len(),
            total_open_orders: open_orders.len(),
            total_exposure,
            instruments,
            risk_limits: RiskLimitsInfo::from(limits.as_ref()),
        }
    }

    /// Calculate exposure for a specific instrument
    pub fn calculate_instrument_exposure(
        instrument: &str,
        positions: &DashMap<String, Position>,
        open_orders: &DashMap<OrderId, Quantity>,
        limits: &Arc<RiskLimits>,
    ) -> Option<InstrumentExposure> {
        let position = positions.get(instrument)?;
        let position_abs = position.quantity.unsigned_abs();

        let position_utilization = if limits.max_position_size.0 > 0 {
            (position_abs as f64 / limits.max_position_size.0 as f64) * 100.0
        } else {
            0.0
        };

        // Count open orders (simplified)
        let open_orders_count = open_orders.len();

        Some(InstrumentExposure {
            instrument: instrument.to_string(),
            position: position.quantity,
            position_abs,
            position_utilization,
            open_orders_count,
        })
    }
}
