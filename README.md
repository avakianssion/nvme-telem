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
[OK] Found 1 controller(s): ["nvme0"]


============================================================
Testing: nvme0
============================================================

Test 2: Reading SMART log... [OK] Success!
  Temperature: 306 K
  Power Cycles: 5603
  Power On Hours: 7129
  Data Written: 132586097 (thousands of 512-byte units)
  Critical Warning: 0

Test 3: Reading controller identity... [OK] Success!

  Identity:
    Vendor ID: 0x144d
    Subsystem Vendor ID: 0x144d
    Serial: S4EMNF0N168983
    Model: SAMSUNG MZVLB1T0HBLR-000L7
    Firmware: 5M2QEXF7

  Capacity:
    Total NVM: 1024209543168 bytes (1024 GB)

  Thermal:
    Warning Temp: 357 K
    Critical Temp: 358 K

  Limits:
    Max Data Transfer Size: 2^9 pages
    Number of Namespaces: 1
    Max Outstanding Commands: 0

Test 4: Reading error log (0x01)... [OK] Success!
  No errors recorded - healthy drive!

Test 5: Reading OCP extended SMART log (0xC0)... [FAIL] Not available: Device does not support OCP extended  S.M.A.R.T. log (invalid GUID)
       (OCP extended SMART is vendor-specific - not all drives support it)


============================================================
Sanity check complete!
============================================================
```
