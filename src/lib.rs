//! # nvme-telem
//!
//! A Rust library for collecting NVMe telemetry and S.M.A.R.T. data on Linux systems.
//!
//! This library provides a type-safe interface to NVMe device telemetry, including
//! S.M.A.R.T./Health logs, error logs, and detailed controller identification data.
//! Data is organized into logical categories (Identity, Capacity, Thermals, etc.)
//! and supports serialization via serde.
//!
//! ## Features
//!
//! - **S.M.A.R.T. Log Reading**: Temperature, power cycles, data written, and health metrics
//! - **Error Log Reading**: Complete error history with detailed diagnostic information
//! - **Controller Identification**: Vendor info, model, serial number, firmware version
//! - **Organized Data Model**: Separate structs for different metric categories
//! - **Type-Safe API**: Strongly typed Rust structs instead of raw byte arrays
//!
//! ## Requirements
//!
//! - Linux operating system with NVMe device support
//! - Root/sudo privileges to access `/dev/nvme*` device files
//! - NVMe storage devices present in the system
//!
//! ## Safety
//!
//! This library uses `unsafe` code to make ioctl system calls to NVMe devices.
//! The unsafe code is isolated to the device communication layer and follows
//! the NVMe specification for command formatting.
pub mod nvme;
