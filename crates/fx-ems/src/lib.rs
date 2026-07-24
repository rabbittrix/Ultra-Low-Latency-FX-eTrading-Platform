//! FX Execution Management System domain components.

pub mod ems;
pub mod strategy;

pub use ems::{EmsEngine, ExecutionDecision, ExecutionDestination};
pub use strategy::{ExecutionAlgo, SlicePlan};
