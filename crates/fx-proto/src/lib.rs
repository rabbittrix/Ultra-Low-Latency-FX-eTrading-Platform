//! Protocol buffer definitions and generated code for the FX eTrading platform
//!
//! This crate contains the gRPC service definitions and message types
//! used for inter-service communication.
//!
//! Note: This requires `protoc` to be installed for the build script to run.
//! The build script generates Rust code from `proto/fx.proto` into `src/generated/`,
//! which is then included here explicitly to avoid OUT_DIR issues in Rust Analyzer.

// Use explicit file path to avoid OUT_DIR issues in Rust Analyzer
// The generated file name is based on the package name (fx.etrading), not the proto file name
pub mod fx {
    pub mod etrading {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/generated/fx.etrading.rs"
        ));
    }
}
