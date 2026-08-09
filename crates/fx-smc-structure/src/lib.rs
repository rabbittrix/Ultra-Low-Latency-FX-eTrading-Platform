//! SMC market structure: swings, equal levels, trendlines, sessions, BOS/CHoCH, FVG.
//!
//! Prices are fixed-point ticks; slopes use rational `i64`/`i128` math (ADR-0002).

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod bos;
pub mod equal;
pub mod fvg;
pub mod geom;
pub mod session;
pub mod swing;
pub mod trendline;

pub use bos::{detect_structure_breaks, StructureBias, StructureBreak, StructureBreakKind};
pub use equal::{cluster_equal_levels, EqualCluster, EqualKind};
pub use fvg::{detect_fvgs, FairValueGap, FvgSide};
pub use geom::{atr_proxy_ticks, equal_tolerance_ticks, project_price};
pub use session::{scan_session_levels, SessionLevels, SessionSnapshot};
pub use swing::{detect_swings, SwingKind, SwingPoint};
pub use trendline::{detect_trendlines, Trendline, TrendlineSide};
