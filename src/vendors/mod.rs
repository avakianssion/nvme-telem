//! Vendor-specific NVMe extensions and telemetry.
//!
//! This module provides support for vendor-specific NVMe log pages and features
//! that extend beyond the standard NVMe specification. Different vendors implement
//! proprietary telemetry and diagnostic features that provide additional insights
//! into drive health and performance.

mod ocp;

// Re-export OCP types and functions
pub use ocp::*;

