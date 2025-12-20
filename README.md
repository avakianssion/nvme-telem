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

## Example

```shell
$: sudo cargo build --example sanity_check
$: sudo cargo run --example sanity_check

--- NVMe Sanity Check ---

Test 1: Discovering NVMe controllers...
✅ Found 1 controller(s): ["nvme0"]


============================================================
Testing: nvme0
============================================================

Test 2: Reading SMART log... ✅ Success!
  Temperature: xxx K
  Power Cycles: xxx
  Power On Hours: xxxxx
  Data Written: xxxxxxx (thousands of 512-byte units)
  Critical Warning: 0

Test 3: Reading controller identity... ✅ Success!

  Identity:
    Vendor ID: 0x144d
    Subsystem Vendor ID: 0x144d
    Serial: Sxxxxxx
    Model: SAMSUNG Mxxxxxx
    Firmware: 5xxxxx

  Capacity:
    Total NVM: xxxxxxxxxxx bytes (xxx GB)

  Thermal:
    Warning Temp: xxx K
    Critical Temp: xxx K

  Limits:
    Max Data Transfer Size: 2^9 pages
    Number of Namespaces: 1
    Max Outstanding Commands: 0

Test 4: Reading OCP extended SMART log (0xC0)... ❌ Not available: Device does not support OCP extended  S.M.A.R.T. log (invalid GUID)
   (OCP extended SMART is vendor-specific - not all drives support it)


============================================================
Sanity check complete!
============================================================
```