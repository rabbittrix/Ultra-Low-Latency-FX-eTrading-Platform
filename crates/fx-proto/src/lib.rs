//! Protocol buffer definitions and generated code for the FX eTrading platform
//!
//! This crate contains the gRPC service definitions and message types
//! used for inter-service communication.

pub mod fx {
    tonic::include_proto!("fx.etrading");
}

pub use fx::*;
