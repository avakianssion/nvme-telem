# nvme-telem

[![Crates.io](https://img.shields.io/crates/v/nvme-telem.svg)](https://crates.io/crates/nvme-telem)

A Rust library for collecting NVMe telemetry and S.M.A.R.T. data on Linux.

## Features

- Read NVMe SMART/Health logs
- Extract detailed controller identification data
- Organized structs for different metric categories (Identity, Capacity, Thermals, etc.)

## Requirements

- Linux operating system
- NVMe storage devices
- Root/sudo privileges to access `/dev/nvme*` devices

## Safety

This library uses unsafe code to make ioctl calls to NVMe devices. All unsafe code is isolated in the device access functions.
