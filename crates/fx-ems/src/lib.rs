//! FX Execution Management System domain components.

pub mod ems;
pub mod strategy;

pub use ems::{ExecutionDecision, ExecutionDestination, EmsEngine};
pub use strategy::{ExecutionAlgo, SlicePlan};
